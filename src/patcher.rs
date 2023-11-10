//! Patcher for typical standalone vbs issues

use regex::Regex;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PatchType {
    DropTarget,
    StandupTarget,
}
impl Display for PatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatchType::DropTarget => write!(f, "DTArray fix"),
            PatchType::StandupTarget => write!(f, "STArray fix"),
        }
    }
}

pub fn patch_vbs_file(vbs_path: &Path) -> io::Result<HashSet<PatchType>> {
    // TODO we probably need to ensure proper encoding here in stead of going for utf8
    let mut file = File::open(&vbs_path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let (patched_text, applied) = patch_script(text);

    let mut file = File::create(&vbs_path)?;
    file.write_all(patched_text.as_bytes())?;
    Ok(applied)
}

pub fn patch_script(script: String) -> (String, HashSet<PatchType>) {
    // TODO we could work with regex::bytes::Regex instead to avoid the conversion to utf8

    let mut applied_patches = HashSet::new();
    let mut patched_script = script;

    if patched_script.contains("DTArray(i)(0)") {
        applied_patches.insert(PatchType::DropTarget);
        patched_script = patch_drop_target_array(patched_script);
    }

    if patched_script.contains("STArray(i)(0)") {
        applied_patches.insert(PatchType::StandupTarget);
        patched_script = patch_standup_target_array(patched_script);
    }

    //todo!("implement patching");
    (patched_script, applied_patches)
}

fn patch_standup_target_array(script: String) -> String {
    let marker = "'Define a variable for each stand-up target";
    let st_class = include_str!("assets/standup_target_class.vbs");
    let mut patched_script = script.replace(marker, format!("{}\r\n{}", st_class, marker).as_str());

    // apply the following replacements

    // ST41 = Array(sw41, Target_Rect_Fat_011_BM_Lit_Room, 41, 0)
    // becomes
    // Set ST41 = (new StandupTarget)(sw41, Target_Rect_Fat_011_BM_Lit_Room, 41, 0)
    let re = Regex::new(r"(ST\d+\s*=\s*)Array\(").unwrap();
    patched_script = re
        .replace_all(&patched_script, |caps: &regex::Captures| {
            let ind = caps.get(1).unwrap().as_str();
            format!("Set {}(new StandupTarget)(", ind)
        })
        .to_string();

    patched_script = patched_script.replace("STArray(i)(0)", "STArray(i).primary");
    patched_script = patched_script.replace("STArray(i)(1)", "STArray(i).prim");
    patched_script = patched_script.replace("STArray(i)(2)", "STArray(i).sw");
    patched_script = patched_script.replace("STArray(i)(3)", "STArray(i).animate");
    patched_script
}

fn patch_drop_target_array(script: String) -> String {
    let marker = "'Define a variable for each drop target";
    let dt_class = include_str!("assets/drop_target_class.vbs");
    let mut patched_script = script.replace(marker, format!("{}\r\n{}", dt_class, marker).as_str());

    // DT7 = Array(dt1, dt1a, pdt1, 7, 0)
    // DT27 = Array(dt2, dt2a, pdt2, 27, 0, false)
    // becomes
    // Set DT7 = (new DropTarget)(dt1, dt1a, pdt1, 7, 0, false)
    // Set DT27 = (new DropTarget)(dt2, dt2a, pdt2, 27, 0, false)
    let re = Regex::new(r"(DT\d+\s*=\s*)Array\((.*?)\s*(,\s*(false|true))?\)").unwrap();
    patched_script = re
        .replace_all(&patched_script, |caps: &regex::Captures| {
            let ind = caps.get(1).unwrap().as_str();
            let ind2 = caps.get(2).unwrap().as_str();
            let ind3 = caps.get(3);
            let false_true = match ind3 {
                Some(c) => c.as_str().to_string(),
                None => ", false".to_string(),
            };
            format!("Set {}(new DropTarget)({}{})", ind, ind2, false_true)
        })
        .to_string();

    patched_script = patched_script.replace("DTArray(i)(0)", "DTArray(i).primary");
    patched_script = patched_script.replace("DTArray(i)(1)", "DTArray(i).secondary");
    patched_script = patched_script.replace("DTArray(i)(2)", "DTArray(i).prim");
    patched_script = patched_script.replace("DTArray(i)(3)", "DTArray(i).sw");
    patched_script = patched_script.replace("DTArray(i)(4)", "DTArray(i).animate");

    // TODO we could work with a regex to catch all cases
    patched_script = patched_script.replace("DTArray(i)(5)", "DTArray(i).isDropped");
    patched_script = patched_script.replace("DTArray(ind)(5)", "DTArray(ind).isDropped");
    patched_script
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_vbs_patch() {
        let script = r#"
'Define a variable for each drop target
Dim DT9, DT47

DT9 = Array(sw9, sw9a, sw9p, 9, 0, false)
DT47 = Array(sw47, sw47a, sw47p, 47, 0)

Sub DoDTAnim()
	Dim i
	For i=0 to Ubound(DTArray)
		DTArray(i)(4) = DTAnimate(DTArray(i)(0),DTArray(i)(1),DTArray(i)(2),DTArray(i)(3),DTArray(i)(4))
	Next
End Sub

'Define a variable for each stand-up target
Dim ST41, ST42

ST41 = Array(sw41, Target_Rect_Fat_011_BM_Lit_Room, 41, 0)
ST42 = Array(sw42, Target_Rect_Fat_010_BM_Lit_Room, 42, 0)

Sub DoSTAnim()
	Dim i
	For i=0 to Ubound(STArray)
		STArray(i)(3) = STAnimate(STArray(i)(0),STArray(i)(1),STArray(i)(2),STArray(i)(3))
	Next
End Sub
"#;
        // vbs files should have windows line endings
        let script = script.replace("\n", "\r\n");

        let expected = r#"
Class DropTarget
  Private m_primary, m_secondary, m_prim, m_sw, m_animate, m_isDropped

  Public Property Get Primary(): Set Primary = m_primary: End Property
  Public Property Let Primary(input): Set m_primary = input: End Property

  Public Property Get Secondary(): Set Secondary = m_secondary: End Property
  Public Property Let Secondary(input): Set m_secondary = input: End Property

  Public Property Get Prim(): Set Prim = m_prim: End Property
  Public Property Let Prim(input): Set m_prim = input: End Property

  Public Property Get Sw(): Sw = m_sw: End Property
  Public Property Let Sw(input): m_sw = input: End Property

  Public Property Get Animate(): Animate = m_animate: End Property
  Public Property Let Animate(input): m_animate = input: End Property

  Public Property Get IsDropped(): IsDropped = m_isDropped: End Property
  Public Property Let IsDropped(input): m_isDropped = input: End Property

  Public default Function init(primary, secondary, prim, sw, animate, isDropped)
    Set m_primary = primary
    Set m_secondary = secondary
    Set m_prim = prim
    m_sw = sw
    m_animate = animate
    m_isDropped = isDropped

    Set Init = Me
  End Function
End Class

'Define a variable for each drop target
Dim DT9, DT47

Set DT9 = (new DropTarget)(sw9, sw9a, sw9p, 9, 0, false)
Set DT47 = (new DropTarget)(sw47, sw47a, sw47p, 47, 0, false)

Sub DoDTAnim()
	Dim i
	For i=0 to Ubound(DTArray)
		DTArray(i).animate = DTAnimate(DTArray(i).primary,DTArray(i).secondary,DTArray(i).prim,DTArray(i).sw,DTArray(i).animate)
	Next
End Sub

Class StandupTarget
  Private m_primary, m_prim, m_sw, m_animate

  Public Property Get Primary(): Set Primary = m_primary: End Property
  Public Property Let Primary(input): Set m_primary = input: End Property

  Public Property Get Prim(): Set Prim = m_prim: End Property
  Public Property Let Prim(input): Set m_prim = input: End Property

  Public Property Get Sw(): Sw = m_sw: End Property
  Public Property Let Sw(input): m_sw = input: End Property

  Public Property Get Animate(): Animate = m_animate: End Property
  Public Property Let Animate(input): m_animate = input: End Property

  Public default Function init(primary, prim, sw, animate)
    Set m_primary = primary
    Set m_prim = prim
    m_sw = sw
    m_animate = animate

    Set Init = Me
  End Function
End Class

'Define a variable for each stand-up target
Dim ST41, ST42

Set ST41 = (new StandupTarget)(sw41, Target_Rect_Fat_011_BM_Lit_Room, 41, 0)
Set ST42 = (new StandupTarget)(sw42, Target_Rect_Fat_010_BM_Lit_Room, 42, 0)

Sub DoSTAnim()
	Dim i
	For i=0 to Ubound(STArray)
		STArray(i).animate = STAnimate(STArray(i).primary,STArray(i).prim,STArray(i).sw,STArray(i).animate)
	Next
End Sub
"#;
        // vbs files should have windows line endings
        let expected = expected.replace("\n", "\r\n");

        let (result, applied) = patch_script(script.to_string());

        assert_eq!(
            applied,
            HashSet::from([PatchType::DropTarget, PatchType::StandupTarget])
        );
        assert_eq!(expected, result);
    }
}