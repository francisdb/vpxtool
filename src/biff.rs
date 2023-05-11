// strings in the vpx file seem to be encoded where each char is suffixed with a zero
// the string has each char suffixed with a zero
// not sure what format this is (biff?)
// https://github.com/vpinball/vpinball/blob/c3c59e09ed56a69759280867affa1f0abf537451/pintable.cpp#L3117
// https://github.com/freezy/VisualPinball.Engine/blob/ec1e9765cd4832c134e889d6e6d03320bc404bd5/VisualPinball.Engine/IO/BiffUtil.cs#L57

pub fn biff_to_utf8(buffer: Vec<u8>) -> Vec<u8> {
    // remove each second byte from the stream
    // this is probably not the best way to do this
    // but it works for now
    let uneven_chars = buffer
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, b)| *b)
        .collect::<Vec<_>>();
    uneven_chars
}
