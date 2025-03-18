// /**
//  * @file main.cpp
//  * @brief ASAP-CABINET-FE in C++/SDL2
//  *
//  * This application scans the VPX_TABLES_PATH recursively for .vpx files and loads the corresponding
//  * images and videos for the playfield, wheel, backglass, and DMD. It creates two windows:
//  * a primary window displaying the table name, wheel, and playfield, and a secondary window
//  * displaying the backglass and DMD. Users can change tables using the left/right arrow or shift keys
//  * with a fade transition, and launch the table via vpinballx_gl by pressing Enter. The application
//  * uses VLC for video playback, handles video context cleanup and setup, loads textures with fallback
//  * options, and renders text using SDL_ttf. Settings are configured via a config.ini file.
//  *
//  * Dependencies:
//  * - sudo apt-get install -y build-essential libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev libvlc-dev
//  *
//  * Compile:
//  * - g++ main.cpp -std=c++17 -I/usr/include/SDL2 -D_REENTRANT -lSDL2 -lSDL2_image -lSDL2_ttf -lSDL2_mixer -lvlc -o ASAPCabinetFE
//  *
//  * Author: Tarso Galvão Mar/2025 | github.com/surtarso/ASAPCabinetFE
//  */
//
// #include <algorithm>    // For std::sort
// #include <SDL.h>        // For SDL main library
// #include <SDL_image.h>  // For SDL image loading
// #include <SDL_ttf.h>    // For SDL font rendering
// #include <SDL_mixer.h>  // For SDL audio mixing
// #include <vlc/vlc.h>    // For VLC video playback
// #include <iostream>     // For standard I/O operations
// #include <filesystem>   // For filesystem operations
// #include <vector>       // For std::vector
// #include <string>       // For std::string
// #include <cstdlib>      // For std::system
// #include <fstream>      // For file I/O operations
// #include <map>          // For std::map container
// #include <memory>       // For std::unique_ptr
// #include <cctype>       // for toupper
//
// namespace fs = std::filesystem; // Alias for filesystem lib
//
// // -------------------------------------------------------------
// // ------------------ Configuration Constants ------------------
//
// // Configuration Variables
// std::string VPX_TABLES_PATH;
// std::string VPX_EXECUTABLE_CMD;
// std::string VPX_SUB_CMD;
// std::string VPX_START_ARGS;
// std::string VPX_END_ARGS;
//
// std::string DEFAULT_TABLE_IMAGE;
// std::string DEFAULT_BACKGLASS_IMAGE;
// std::string DEFAULT_DMD_IMAGE;
// std::string DEFAULT_WHEEL_IMAGE;
// std::string DEFAULT_TABLE_VIDEO;
// std::string DEFAULT_BACKGLASS_VIDEO;
// std::string DEFAULT_DMD_VIDEO;
//
// std::string CUSTOM_TABLE_IMAGE;
// std::string CUSTOM_BACKGLASS_IMAGE;
// std::string CUSTOM_DMD_IMAGE;
// std::string CUSTOM_WHEEL_IMAGE;
// std::string CUSTOM_TABLE_VIDEO;
// std::string CUSTOM_BACKGLASS_VIDEO;
// std::string CUSTOM_DMD_VIDEO;
//
// int MAIN_WINDOW_MONITOR;
// int MAIN_WINDOW_WIDTH;
// int MAIN_WINDOW_HEIGHT;
// int WHEEL_IMAGE_SIZE;
// int WHEEL_IMAGE_MARGIN;
// std::string FONT_PATH;
// int FONT_SIZE;
//
// int SECOND_WINDOW_MONITOR;
// int SECOND_WINDOW_WIDTH;
// int SECOND_WINDOW_HEIGHT;
// int BACKGLASS_MEDIA_WIDTH;
// int BACKGLASS_MEDIA_HEIGHT;
// int DMD_MEDIA_WIDTH;
// int DMD_MEDIA_HEIGHT;
//
// int FADE_DURATION_MS;
// Uint8 FADE_TARGET_ALPHA;
// std::string TABLE_CHANGE_SOUND;
// std::string TABLE_LOAD_SOUND;
//
// // --------------------------------------------------------------
// // ---------------------- Data Structures -----------------------
//
// // Structure to hold information about each pinball table
// struct Table {
//     std::string tableName;       // Name of the table
//     std::string vpxFile;         // Path to the .vpx file
//     std::string folder;          // Folder containing the table's assets
//     std::string tableImage;      // Path to the table image
//     std::string wheelImage;      // Path to the wheel image
//     std::string backglassImage;  // Path to the backglass image
//     std::string dmdImage;        // Path to the DMD image
//     std::string tableVideo;      // Path to the table video
//     std::string backglassVideo;  // Path to the backglass video
//     std::string dmdVideo;        // Path to the DMD video
// };
//
// std::map<char, int> letterIndex; // Declare letterIndex
//
// // Structure to hold video context information for VLC video playback
// struct VideoContext {
//     SDL_Texture* texture;  // SDL texture to render the video frame
//     Uint8* pixels;         // Pointer to the pixel data of the video frame
//     int pitch;             // Number of bytes in a row of pixel data
//     SDL_mutex* mutex;      // Mutex to synchronize access to the pixel data
//     int width;             // Width of the video frame
//     int height;            // Height of the video frame
//     bool updated;          // Flag to indicate if the video frame has been updated
// };
//
// // -----------------------------------------------------------------------------------
// // --------------------------------- Utility Functions -------------------------------
// // Get image path with fallback
// std::string getImagePath(const std::string &root, const std::string &imagePath, const std::string &defaultImagePath) {
//     fs::path imageFile = fs::path(root) / imagePath;
//     if (fs::exists(imageFile))
//         return imageFile.string();
//     return defaultImagePath;
// }
//
// // Get video path with fallback
// std::string getVideoPath(const std::string &root, const std::string &videoPath, const std::string &defaultVideoPath) {
//     fs::path videoFile = fs::path(root) / videoPath;
//     if (fs::exists(videoFile))
//         return videoFile.string();
//     else if (fs::exists(defaultVideoPath))
//         return defaultVideoPath;
//     else
//         return "";
// }
//
// // ------------------- Load table list --------------------
// /**
//  * @brief Loads a list of tables from the directory specified by VPX_TABLES_PATH.
//  *
//  * This function recursively iterates through the directory specified by VPX_TABLES_PATH,
//  * looking for files with the ".vpx" extension. For each valid file found, it creates a
//  * Table object and populates its fields with relevant information such as file paths
//  * for images and videos. The list of tables is then sorted by table name.
//  *
//  * @return std::vector<Table> A sorted vector of Table objects.
//  */
// std::vector<Table> loadTableList() {
//     std::vector<Table> tables;
//     for (const auto &entry : fs::recursive_directory_iterator(VPX_TABLES_PATH)) {
//         if (entry.is_regular_file() && entry.path().extension() == ".vpx") {
//             Table table;
//             table.vpxFile = entry.path().string();
//             table.folder = entry.path().parent_path().string();
//             table.tableName = entry.path().stem().string();
//             table.tableImage     = getImagePath(table.folder, CUSTOM_TABLE_IMAGE, DEFAULT_TABLE_IMAGE);
//             table.wheelImage     = getImagePath(table.folder, CUSTOM_WHEEL_IMAGE, DEFAULT_WHEEL_IMAGE);
//             table.backglassImage = getImagePath(table.folder, CUSTOM_BACKGLASS_IMAGE, DEFAULT_BACKGLASS_IMAGE);
//             table.dmdImage       = getImagePath(table.folder, CUSTOM_DMD_IMAGE, DEFAULT_DMD_IMAGE);
//             table.tableVideo     = getVideoPath(table.folder, CUSTOM_TABLE_VIDEO, DEFAULT_TABLE_VIDEO);
//             table.backglassVideo = getVideoPath(table.folder, CUSTOM_BACKGLASS_VIDEO, DEFAULT_BACKGLASS_VIDEO);
//             table.dmdVideo       = getVideoPath(table.folder, CUSTOM_DMD_VIDEO, DEFAULT_DMD_VIDEO);
//             tables.push_back(table);
//         }
//     }
//     std::sort(tables.begin(), tables.end(), [](const Table &a, const Table &b) {
//         return a.tableName < b.tableName;
//     });
//
//     // Build the letter index after loading and sorting
//     letterIndex.clear(); // Clear the map before building it.
//     for (int i = 0; i < tables.size(); ++i) {
//         char firstLetter = toupper(tables[i].tableName[0]);
//         if (letterIndex.find(firstLetter) == letterIndex.end()) {
//             letterIndex[firstLetter] = i;
//         }
//     }
//
//     return tables;
// }
//
// // ------- Load images texture with fallback ----------
// SDL_Texture* loadTexture(SDL_Renderer* renderer, const std::string &path, const std::string &fallbackPath) {
//     SDL_Texture* tex = IMG_LoadTexture(renderer, path.c_str());
//     if (!tex) {
//         std::cerr << "Failed to load " << path << ". Using fallback." << std::endl;
//         tex = IMG_LoadTexture(renderer, fallbackPath.c_str());
//     }
//     return tex;
// }
//
// // ----------------- Load font --------------
// SDL_Texture* renderText(SDL_Renderer* renderer, TTF_Font* font, const std::string &message, SDL_Color color, SDL_Rect &textRect) {
//     SDL_Surface* surf = TTF_RenderUTF8_Blended(font, message.c_str(), color);
//     if (!surf) {
//         std::cerr << "TTF_RenderUTF8_Blended error: " << TTF_GetError() << std::endl;
//         return nullptr;
//     }
//     SDL_Texture* texture = SDL_CreateTextureFromSurface(renderer, surf);
//     textRect.w = surf->w;
//     textRect.h = surf->h;
//     SDL_FreeSurface(surf);
//     return texture;
// }
//
// // --------------------------------------------------------------------------------
// // ------------------------ Handle video playback (vlc) ---------------------------
//
// // Locks the video context mutex and provides access to the pixel data.
// void* lock(void* data, void** pixels) {
//     VideoContext* ctx = static_cast<VideoContext*>(data);
//     if (!ctx || !ctx->mutex) return nullptr;
//     SDL_LockMutex(ctx->mutex);
//     *pixels = ctx->pixels;
//     return nullptr;
// }
//
// // Unlocks the video context and updates its state.
// void unlock(void* data, void* id, void* const* pixels) {
//     VideoContext* ctx = static_cast<VideoContext*>(data);
//     if (ctx) {
//         ctx->updated = true;
//         SDL_UnlockMutex(ctx->mutex);
//     }
// }
//
// // Empty function needed for display operations.
// void display(void* data, void* id) {}
//
// // Cleans up the resources associated with a VideoContext and libvlc_media_player_t.
// void cleanupVideoContext(VideoContext& ctx, libvlc_media_player_t*& player) {
//     if (player) {
//         libvlc_media_player_stop(player);
//         libvlc_media_player_release(player);
//         player = nullptr;
//     }
//     if (ctx.texture) {
//         SDL_DestroyTexture(ctx.texture);
//         ctx.texture = nullptr;
//     }
//     if (ctx.pixels) {
//         delete[] ctx.pixels;
//         ctx.pixels = nullptr;
//     }
//     if (ctx.mutex) {
//         SDL_DestroyMutex(ctx.mutex);
//         ctx.mutex = nullptr;
//     }
// }
//
// /**
//  * @brief Sets up a video player using libVLC and SDL.
//  *
//  * This function initializes a video player with the specified video file path,
//  * using the provided libVLC instance and SDL renderer. It configures the video
//  * player to loop indefinitely and sets up the necessary SDL texture and pixel
//  * buffer for rendering the video frames.
//  *
//  * @param vlcInstance A pointer to the libVLC instance.
//  * @param renderer A pointer to the SDL renderer.
//  * @param videoPath The file path to the video to be played.
//  * @param ctx A reference to the VideoContext structure to store video-related data.
//  * @param width The width of the video frame.
//  * @param height The height of the video frame.
//  * @return A pointer to the initialized libvlc_media_player_t, or nullptr if an error occurs.
//  */
// libvlc_media_player_t* setupVideoPlayer(libvlc_instance_t* vlcInstance, SDL_Renderer* renderer,
//                                       const std::string& videoPath, VideoContext& ctx, int width, int height) {
//     libvlc_media_t* media = libvlc_media_new_path(vlcInstance, videoPath.c_str());
//     if (!media) {
//         std::cerr << "Failed to create media for " << videoPath << std::endl;
//         return nullptr;
//     }
//
//     libvlc_media_add_option(media, "input-repeat=65535"); // Loop indefinitely
//
//     libvlc_media_player_t* player = libvlc_media_player_new_from_media(media);
//     libvlc_media_release(media);
//     if (!player) {
//         std::cerr << "Failed to create media player for " << videoPath << std::endl;
//         return nullptr;
//     }
//
//     ctx.texture = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888, SDL_TEXTUREACCESS_STREAMING, width, height);
//     if (!ctx.texture) {
//         std::cerr << "Failed to create texture: " << SDL_GetError() << std::endl;
//         libvlc_media_player_release(player);
//         return nullptr;
//     }
//
//     ctx.pixels = new (std::nothrow) Uint8[width * height * 4]; // BGRA: 4 bytes per pixel
//     if (!ctx.pixels) {
//         std::cerr << "Failed to allocate video buffer" << std::endl;
//         SDL_DestroyTexture(ctx.texture);
//         libvlc_media_player_release(player);
//         return nullptr;
//     }
//
//     ctx.pitch = width * 4;           // Set the pitch (number of bytes in a row of pixel data) to width * 4 (BGRA format)
//     ctx.mutex = SDL_CreateMutex();   // Create a mutex to synchronize access to the pixel data
//     ctx.width = width;               // Set the width of the video frame
//     ctx.height = height;             // Set the height of the video frame
//     ctx.updated = false;             // Initialize the updated flag to false
//
//     libvlc_video_set_callbacks(player, lock, unlock, display, &ctx);
//     libvlc_video_set_format(player, "BGRA", width, height, width * 4);
//
//     if (libvlc_media_player_play(player) < 0) {
//         std::cerr << "Failed to play video: " << videoPath << std::endl;
//         cleanupVideoContext(ctx, player);
//         return nullptr;
//     }
//
//     SDL_Delay(100); // Wait for VLC to initialize
//     return player;
// }
//
// // -----------------------------------------------------------------------
// // --------------------------- Launch Table ------------------------------
//
// /**
//  * @brief Launches a VPX table using the specified command arguments.
//  *
//  * This function constructs a command string to launch a VPX table using
//  * predefined command arguments and the file path of the table. It then
//  * outputs the command to the console and executes it using the system
//  * command.
//  *
//  * @param table A reference to a Table object containing the file path
//  *              of the VPX table to be launched.
//  */
// void launchTable(const Table &table) {
//     std::string command = VPX_START_ARGS + " " + VPX_EXECUTABLE_CMD + " " +
//                           VPX_SUB_CMD + " \"" + table.vpxFile + "\" " + VPX_END_ARGS;
//     std::cout << "Launching: " << command << std::endl;
//     std::system(command.c_str());
// }
//
// //-------------------------------------------------------------------------
// // ----------------------------- Settings ---------------------------------
//
// // Helper functions to get values with defaults
// std::string get_string(const std::map<std::string, std::map<std::string, std::string>>& config,
//         const std::string& section, const std::string& key, const std::string& default_value) {
//     if (config.count(section) && config.at(section).count(key)) {
//         return config.at(section).at(key);
//     }
//     return default_value;
// }
//
// int get_int(const std::map<std::string, std::map<std::string, std::string>>& config,
//         const std::string& section, const std::string& key, int default_value) {
//     if (config.count(section) && config.at(section).count(key)) {
//         try {
//             return std::stoi(config.at(section).at(key));
//         } catch (const std::exception&) {
//             return default_value;
//         }
//     }
//     return default_value;
// }
//
// // -------------- Structure to hold all config data --------------
// /**
//  * @brief Loads a configuration file and parses it into a nested map structure.
//  *
//  * This function reads a configuration file specified by the filename parameter.
//  * The file is expected to have sections denoted by square brackets (e.g., [SECTION])
//  * and key-value pairs within those sections (e.g., key=value). Comments and empty
//  * lines are ignored.
//  *
//  * @param filename The path to the configuration file to be loaded.
//  * @return A nested map where the outer map's keys are section names and the inner
//  *         map's keys and values are the key-value pairs within those sections.
//  *
//  * @note If the file cannot be opened, an empty configuration is returned and an
//  *       error message is printed to std::cerr.
//  */
// std::map<std::string, std::map<std::string, std::string>> load_config(const std::string& filename) {
//     std::map<std::string, std::map<std::string, std::string>> config;
//     std::ifstream file(filename);
//     std::string current_section;
//
//     if (!file.is_open()) {
//         std::cerr << "Could not open " << filename << ". Using defaults." << std::endl;
//         return config;
//     }
//
//     std::string line;
//     while (std::getline(file, line)) {
//         // Skip empty lines or comments
//         if (line.empty() || line[0] == ';') continue;
//
//         // Check for section (e.g., [VPX])
//         if (line[0] == '[') {
//             size_t end = line.find(']');
//             if (end != std::string::npos) {
//                 current_section = line.substr(1, end - 1);
//                 config[current_section]; // Create section if it doesn't exist
//             }
//             continue;
//         }
//
//         // Parse key=value pairs
//         size_t eq_pos = line.find('=');
//         if (eq_pos != std::string::npos && !current_section.empty()) {
//             std::string key = line.substr(0, eq_pos);
//             std::string value = line.substr(eq_pos + 1);
//             // Remove any trailing or leading whitespace (basic cleanup)
//             key.erase(0, key.find_first_not_of(" \t"));
//             key.erase(key.find_last_not_of(" \t") + 1);
//             value.erase(0, value.find_first_not_of(" \t"));
//             value.erase(value.find_last_not_of(" \t") + 1);
//             config[current_section][key] = value;
//         }
//     }
//     file.close();
//     return config;
// }
//
// // -----------------------------------------------------------------------
// // --------------------- Initialization Guard Classes --------------------
// /**
//  * @class SDLInitGuard
//  * @brief A guard class to manage the initialization and cleanup of SDL.
//  *
//  * This class initializes SDL with the specified flags upon construction
//  * and ensures that SDL is properly cleaned up upon destruction if the
//  * initialization was successful.
//  *
//  * @public
//  * @var bool success
//  * Indicates whether SDL was successfully initialized.
//  *
//  * @public
//  * @fn SDLInitGuard(Uint32 flags)
//  * @brief Constructs an SDLInitGuard object and initializes SDL.
//  *
//  * @param flags The SDL initialization flags.
//  *
//  * If SDL initialization fails, an error message is printed to std::cerr.
//  *
//  * @public
//  * @fn ~SDLInitGuard()
//  * @brief Destructs the SDLInitGuard object and cleans up SDL.
//  *
//  * If SDL was successfully initialized, SDL_Quit() is called to clean up.
//  */
// class SDLInitGuard {
// public:
//     bool success;
//     SDLInitGuard(Uint32 flags) : success(false) {
//         if (SDL_Init(flags) == 0) {
//             success = true;
//         } else {
//             std::cerr << "SDL_Init Error: " << SDL_GetError() << std::endl;
//         }
//     }
//     ~SDLInitGuard() {
//         if (success) SDL_Quit();
//     }
// };
//
// /**
//  * @class IMGInitGuard
//  * @brief A guard class to manage the initialization and cleanup of the SDL_image library.
//  *
//  * This class ensures that the SDL_image library is properly initialized with the specified flags
//  * and automatically cleaned up when the object goes out of scope.
//  *
//  * @param flags The initialization flags for the SDL_image library.
//  *
//  * The constructor initializes the SDL_image library with the given flags. If the initialization
//  * fails, an error message is printed to std::cerr and the flags are set to 0.
//  *
//  * The destructor cleans up the SDL_image library if it was successfully initialized.
//  */
// class IMGInitGuard {
// public:
//     int flags;
//     IMGInitGuard(int flags) : flags(flags) {
//         if (!(IMG_Init(flags) & flags)) {
//             std::cerr << "IMG_Init Error: " << IMG_GetError() << std::endl;
//             this->flags = 0;
//         }
//     }
//     ~IMGInitGuard() {
//         if (flags) IMG_Quit();
//     }
// };
//
// /**
//  * @class TTFInitGuard
//  * @brief A guard class to manage the initialization and cleanup of the SDL_ttf library.
//  *
//  * This class ensures that the SDL_ttf library is properly initialized when an instance
//  * of the class is created and properly cleaned up when the instance is destroyed.
//  *
//  * @details
//  * The constructor attempts to initialize the SDL_ttf library by calling TTF_Init().
//  * If the initialization is successful, the `success` member is set to true. Otherwise,
//  * an error message is printed to the standard error output.
//  *
//  * The destructor checks if the initialization was successful (i.e., `success` is true).
//  * If so, it calls TTF_Quit() to clean up the SDL_ttf library.
//  *
//  * @note
//  * This class is useful for RAII (Resource Acquisition Is Initialization) to ensure
//  * that the SDL_ttf library is properly managed within a scope.
//  */
// class TTFInitGuard {
// public:
//     bool success;
//     TTFInitGuard() : success(false) {
//         if (TTF_Init() == 0) {
//             success = true;
//         } else {
//             std::cerr << "TTF_Init Error: " << TTF_GetError() << std::endl;
//         }
//     }
//     ~TTFInitGuard() {
//         if (success) TTF_Quit();
//     }
// };
//
// /**
//  * @class MixerGuard
//  * @brief A RAII (Resource Acquisition Is Initialization) class to manage SDL_mixer audio initialization and cleanup.
//  *
//  * This class ensures that the SDL_mixer audio subsystem is properly initialized and closed.
//  * It attempts to open the audio with the specified parameters upon construction and closes
//  * the audio upon destruction if the initialization was successful.
//  *
//  * @param frequency The audio frequency in samples per second (Hz).
//  * @param format The audio format (e.g., AUDIO_S16SYS).
//  * @param channels The number of audio channels (1 for mono, 2 for stereo).
//  * @param chunksize The size of the audio chunks in bytes.
//  *
//  * @var success A boolean flag indicating whether the audio initialization was successful.
//  */
// class MixerGuard {
// public:
//     bool success;
//     MixerGuard(int frequency, Uint16 format, int channels, int chunksize) : success(false) {
//         if (Mix_OpenAudio(frequency, format, channels, chunksize) == 0) {
//             success = true;
//         } else {
//             std::cerr << "SDL_mixer Error: " << Mix_GetError() << std::endl;
//         }
//     }
//     ~MixerGuard() {
//         if (success) Mix_CloseAudio();
//     }
// };
//
// enum class TransitionState { IDLE, FADING_OUT, FADING_IN };
//
//
// /** ------------------------------------------------------------------------------------
//  * @brief Main loop of the application.
//  * -------------------------------------------------------------------------------------
//  * This loop handles the following tasks:
//  * - Polls and processes SDL events, including quitting the application and handling key presses for table navigation and launching.
//  * - Manages transitions between tables, including fading out the current table and fading in the new table.
//  * - Updates the alpha modulation of textures and video contexts based on the transition state.
//  * - Updates video textures if new frames are available.
//  * - Renders the primary screen, including the table playfield, wheel image, and table name.
//  * - Renders the secondary screen, including the backglass and DMD (Dot Matrix Display).
//  * - Delays to control the frame rate.
//  *
//  * The loop continues until the `quit` flag is set to true, typically by an SDL_QUIT event or pressing the escape key.
//  */
// int main(int argc, char* argv[]) {
//     // -----------------------------------------------------------------------
//     // --------------- Load the configuration from config.ini ----------------
//     auto config = load_config("config.ini");
//
//     // Assign VPX settings
//     VPX_TABLES_PATH        = get_string(config, "VPX", "TablesPath", "/home/tarso/Games/vpinball/build/tables/");
//     VPX_EXECUTABLE_CMD     = get_string(config, "VPX", "ExecutableCmd", "/home/tarso/Games/vpinball/build/VPinballX_GL");
//     VPX_SUB_CMD            = get_string(config, "Internal", "SubCmd", "-Play");
//     VPX_START_ARGS         = get_string(config, "VPX", "StartArgs", "");
//     VPX_END_ARGS           = get_string(config, "VPX", "EndArgs", "");
//
//     // Assign CustomMedia settings
//     CUSTOM_TABLE_IMAGE     = get_string(config, "CustomMedia", "TableImage", "images/table.png");
//     CUSTOM_BACKGLASS_IMAGE = get_string(config, "CustomMedia", "BackglassImage", "images/backglass.png");
//     CUSTOM_DMD_IMAGE       = get_string(config, "CustomMedia", "DmdImage", "images/marquee.png");
//     CUSTOM_WHEEL_IMAGE     = get_string(config, "CustomMedia", "WheelImage", "images/wheel.png");
//     CUSTOM_TABLE_VIDEO     = get_string(config, "CustomMedia", "TableVideo", "video/table.mp4");
//     CUSTOM_BACKGLASS_VIDEO = get_string(config, "CustomMedia", "BackglassVideo", "video/backglass.mp4");
//     CUSTOM_DMD_VIDEO       = get_string(config, "CustomMedia", "DmdVideo", "video/dmd.mp4");
//
//     // Assign WindowSettings
//     MAIN_WINDOW_MONITOR    = get_int(config, "WindowSettings", "MainMonitor", 1);
//     MAIN_WINDOW_WIDTH      = get_int(config, "WindowSettings", "MainWidth", 1080);
//     MAIN_WINDOW_HEIGHT     = get_int(config, "WindowSettings", "MainHeight", 1920);
//     SECOND_WINDOW_MONITOR  = get_int(config, "WindowSettings", "SecondMonitor", 0);
//     SECOND_WINDOW_WIDTH    = get_int(config, "WindowSettings", "SecondWidth", 1024);
//     SECOND_WINDOW_HEIGHT   = get_int(config, "WindowSettings", "SecondHeight", 1024);
//
//     // Assign Font settings
//     FONT_PATH              = get_string(config, "Internal", "FontPath", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
//     FONT_SIZE              = get_int(config, "Font", "Size", 28);
//
//     // Assign MediaDimensions
//     WHEEL_IMAGE_SIZE       = get_int(config, "MediaDimensions", "WheelImageSize", 300);
//     WHEEL_IMAGE_MARGIN     = get_int(config, "MediaDimensions", "WheelImageMargin", 24);
//     BACKGLASS_MEDIA_WIDTH  = get_int(config, "MediaDimensions", "BackglassWidth", 1024);
//     BACKGLASS_MEDIA_HEIGHT = get_int(config, "MediaDimensions", "BackglassHeight", 768);
//     DMD_MEDIA_WIDTH        = get_int(config, "MediaDimensions", "DmdWidth", 1024);
//     DMD_MEDIA_HEIGHT       = get_int(config, "MediaDimensions", "DmdHeight", 256);
//
//     // Set defaults for variables
//     DEFAULT_TABLE_IMAGE     = get_string(config, "Internal", "DefaultTableImage", "img/default_table.png");
//     DEFAULT_BACKGLASS_IMAGE = get_string(config, "Internal", "DefaultBackglassImage", "img/default_backglass.png");
//     DEFAULT_DMD_IMAGE       = get_string(config, "Internal", "DefaultDmdImage", "img/default_dmd.png");
//     DEFAULT_WHEEL_IMAGE     = get_string(config, "Internal", "DefaultWheelImage", "img/default_wheel.png");
//     DEFAULT_TABLE_VIDEO     = get_string(config, "Internal", "DefaultTableVideo", "img/default_table.mp4");
//     DEFAULT_BACKGLASS_VIDEO = get_string(config, "Internal", "DefaultBackglassVideo", "img/default_backglass.mp4");
//     DEFAULT_DMD_VIDEO       = get_string(config, "Internal", "DefaultDmdVideo", "img/default_dmd.mp4");
//     FADE_DURATION_MS        = get_int(config, "Internal", "FadeDurationMs", 1);
//     FADE_TARGET_ALPHA       = static_cast<Uint8>(get_int(config, "Internal", "FadeTargetAlpha", 255));
//     TABLE_CHANGE_SOUND      = get_string(config, "Internal", "TableChangeSound", "snd/table_change.mp3");
//     TABLE_LOAD_SOUND        = get_string(config, "Internal", "TableLoadSound", "snd/table_load.mp3");
//
//     // ----------------------------------------------------
//     // ------------------ Initialization ------------------
//
//     // Library initialization guards
//     SDLInitGuard sdlInit(SDL_INIT_VIDEO | SDL_INIT_TIMER | SDL_INIT_AUDIO);
//     if (!sdlInit.success) return 1;
//
//     IMGInitGuard imgInit(IMG_INIT_PNG | IMG_INIT_JPG);
//     if (!imgInit.flags) return 1;
//
//     TTFInitGuard ttfInit;
//     if (!ttfInit.success) return 1;
//
//     MixerGuard mixerGuard(44100, MIX_DEFAULT_FORMAT, 2, 2048);
//     if (!mixerGuard.success) return 1;
//
//     auto vlcInstance = std::unique_ptr<libvlc_instance_t, void(*)(libvlc_instance_t*)>(
//         libvlc_new(0, nullptr), libvlc_release);
//     if (!vlcInstance) {
//         std::cerr << "Failed to initialize VLC instance." << std::endl;
//         return 1;
//     }
//
//     auto primaryWindow = std::unique_ptr<SDL_Window, void(*)(SDL_Window*)>(
//         SDL_CreateWindow("Playfield",
//             SDL_WINDOWPOS_CENTERED_DISPLAY(MAIN_WINDOW_MONITOR), SDL_WINDOWPOS_CENTERED,
//             MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT, SDL_WINDOW_SHOWN | SDL_WINDOW_BORDERLESS),
//         SDL_DestroyWindow);
//     if (!primaryWindow) {
//         std::cerr << "Failed to create primary window: " << SDL_GetError() << std::endl;
//         return 1;
//     }
//
//     auto primaryRenderer = std::unique_ptr<SDL_Renderer, void(*)(SDL_Renderer*)>(
//         SDL_CreateRenderer(primaryWindow.get(), -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC),
//         SDL_DestroyRenderer);
//     if (!primaryRenderer) {
//         std::cerr << "Failed to create primary renderer: " << SDL_GetError() << std::endl;
//         return 1;
//     }
//
//     auto secondaryWindow = std::unique_ptr<SDL_Window, void(*)(SDL_Window*)>(
//         SDL_CreateWindow("Backglass",
//             SDL_WINDOWPOS_CENTERED_DISPLAY(SECOND_WINDOW_MONITOR), SDL_WINDOWPOS_CENTERED,
//             SECOND_WINDOW_WIDTH, SECOND_WINDOW_HEIGHT, SDL_WINDOW_SHOWN | SDL_WINDOW_BORDERLESS),
//         SDL_DestroyWindow);
//     if (!secondaryWindow) {
//         std::cerr << "Failed to create secondary window: " << SDL_GetError() << std::endl;
//         return 1;
//     }
//
//     auto secondaryRenderer = std::unique_ptr<SDL_Renderer, void(*)(SDL_Renderer*)>(
//         SDL_CreateRenderer(secondaryWindow.get(), -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC),
//         SDL_DestroyRenderer);
//     if (!secondaryRenderer) {
//         std::cerr << "Failed to create secondary renderer: " << SDL_GetError() << std::endl;
//         return 1;
//     }
//
//     auto font = std::unique_ptr<TTF_Font, void(*)(TTF_Font*)>(
//         TTF_OpenFont(FONT_PATH.c_str(), FONT_SIZE), TTF_CloseFont);
//     if (!font) {
//         std::cerr << "Failed to load font: " << TTF_GetError() << std::endl;
//     }
//
//     auto tableChangeSound = std::unique_ptr<Mix_Chunk, void(*)(Mix_Chunk*)>(
//         Mix_LoadWAV(TABLE_CHANGE_SOUND.c_str()), Mix_FreeChunk);
//     if (!tableChangeSound) {
//         std::cerr << "Mix_LoadWAV Error: " << Mix_GetError() << std::endl;
//     }
//
//     auto tableLoadSound = std::unique_ptr<Mix_Chunk, void(*)(Mix_Chunk*)>(
//         Mix_LoadWAV(TABLE_LOAD_SOUND.c_str()), Mix_FreeChunk);
//     if (!tableLoadSound) {
//         std::cerr << "Mix_LoadWAV Error: " << Mix_GetError() << std::endl;
//     }
//
//     std::vector<Table> tables = loadTableList();
//     if (tables.empty()) {
//         std::cerr << "Edit config.ini, no .vpx files found in " << VPX_TABLES_PATH << std::endl;
//         return 1;
//     }
//
//     // ------------------ Texture and State Variables ------------------
//
//     // Index of the currently selected table
//     size_t currentIndex = 0;
//
//     // SDL textures for rendering images and videos
//     SDL_Texture* tableTexture = nullptr;       // Texture for the table playfield image or video
//     SDL_Texture* wheelTexture = nullptr;       // Texture for the wheel image
//     SDL_Texture* backglassTexture = nullptr;   // Texture for the backglass image or video
//     SDL_Texture* dmdTexture = nullptr;         // Texture for the DMD image or video
//     SDL_Texture* tableNameTexture = nullptr;   // Texture for rendering the table name text
//
//     // Rectangle to define the position and size of the table name text
//     SDL_Rect tableNameRect = {0, 0, 0, 0};
//
//     // VLC media player instances for table, backglass, and DMD videos
//     libvlc_media_player_t* tableVideoPlayer = nullptr;
//     libvlc_media_player_t* backglassVideoPlayer = nullptr;
//     libvlc_media_player_t* dmdVideoPlayer = nullptr;
//
//     // Video context structures to hold rendering data for table, backglass, and DMD videos
//     VideoContext tableVideoCtx = {nullptr, nullptr, 0, nullptr, 0, 0, false};
//     VideoContext backglassVideoCtx = {nullptr, nullptr, 0, nullptr, 0, 0, false};
//     VideoContext dmdVideoCtx = {nullptr, nullptr, 0, nullptr, 0, 0, false};
//
//     // -----------------------------------------------------------------------------------
//     // -------------------------- Load Current Table Textures ----------------------------
//     /**
//      * @brief Loads the current table textures and video players.
//      *
//      * This function performs the following tasks:
//      * - Cleans up existing video contexts and textures.
//      * - Loads the video or image textures for the table, backglass, and DMD (Dot Matrix Display).
//      * - Loads the wheel image texture.
//      * - Renders the table name text texture if a font is available.
//      *
//      * The function uses the current index to retrieve the table information from the `tables` array.
//      *
//      * @note This function assumes that `cleanupVideoContext`, `setupVideoPlayer`, `loadTexture`, and `renderText`
//      * are defined elsewhere in the codebase.
//      */
//     auto loadCurrentTableTextures = [&]() {
//         // Cleanup
//         cleanupVideoContext(tableVideoCtx, tableVideoPlayer);
//         cleanupVideoContext(backglassVideoCtx, backglassVideoPlayer);
//         cleanupVideoContext(dmdVideoCtx, dmdVideoPlayer);
//
//         if (tableTexture)     { SDL_DestroyTexture(tableTexture); tableTexture = nullptr; }
//         if (backglassTexture) { SDL_DestroyTexture(backglassTexture); backglassTexture = nullptr; }
//         if (dmdTexture)       { SDL_DestroyTexture(dmdTexture); dmdTexture = nullptr; }
//         if (tableNameTexture) { SDL_DestroyTexture(tableNameTexture); tableNameTexture = nullptr; }
//
//         // Start Loading
//         const Table &tbl = tables[currentIndex]; // Structure to hold information about each pinball table
//
//         if (!tbl.tableVideo.empty()) {
//             tableVideoPlayer = setupVideoPlayer(vlcInstance.get(), primaryRenderer.get(),
//                                                 tbl.tableVideo, tableVideoCtx,
//                                                 MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT);
//         } else {
//             tableTexture = loadTexture(primaryRenderer.get(), tbl.tableImage, DEFAULT_TABLE_IMAGE);
//         }
//
//         if (!tbl.backglassVideo.empty()) {
//             backglassVideoPlayer = setupVideoPlayer(vlcInstance.get(), secondaryRenderer.get(),
//                                                     tbl.backglassVideo, backglassVideoCtx,
//                                                     BACKGLASS_MEDIA_WIDTH, BACKGLASS_MEDIA_HEIGHT);
//         } else {
//             backglassTexture = loadTexture(secondaryRenderer.get(), tbl.backglassImage, DEFAULT_BACKGLASS_IMAGE);
//         }
//
//         if (!tbl.dmdVideo.empty()) {
//             dmdVideoPlayer = setupVideoPlayer(vlcInstance.get(), secondaryRenderer.get(),
//                                             tbl.dmdVideo, dmdVideoCtx,
//                                             DMD_MEDIA_WIDTH, DMD_MEDIA_HEIGHT);
//         } else {
//             dmdTexture = loadTexture(secondaryRenderer.get(), tbl.dmdImage, DEFAULT_DMD_IMAGE);
//         }
//
//         wheelTexture = loadTexture(primaryRenderer.get(), tbl.wheelImage, DEFAULT_WHEEL_IMAGE);
//
//         if (font) {
//             SDL_Color textColor = {255, 255, 255, 255};
//             tableNameTexture = renderText(primaryRenderer.get(), font.get(), tbl.tableName, textColor, tableNameRect);
//             tableNameRect.x = 10;
//             tableNameRect.y = MAIN_WINDOW_HEIGHT - tableNameRect.h - 20;
//         }
//     };
//
//     loadCurrentTableTextures();
//
//     // ------- Transition and Key Event Variables
//     TransitionState transitionState = TransitionState::IDLE; // Current state of the transition (idle, fading out, fading in)
//     Uint32 transitionStartTime = 0; // Timestamp when the transition started
//     bool quit = false; // Flag to indicate if the application should quit
//     SDL_Event event; // SDL event structure to handle events
//
//     auto performTableTransition = [&]() {
//         if (tableVideoPlayer) libvlc_media_player_stop(tableVideoPlayer);
//         if (backglassVideoPlayer) libvlc_media_player_stop(backglassVideoPlayer);
//         if (dmdVideoPlayer) libvlc_media_player_stop(dmdVideoPlayer);
//         if (tableChangeSound) Mix_PlayChannel(-1, tableChangeSound.get(), 0);
//         transitionState = TransitionState::FADING_OUT;
//         transitionStartTime = SDL_GetTicks();
//     };
//     //--------------------------------------------------------------------------
//     // ------------------------------ Main loop --------------------------------
//     while (!quit) {
//         // Key Events
//         while (SDL_PollEvent(&event)) {
//             if (event.type == SDL_QUIT) {
//                 quit = true;
//             }
//             else if (event.type == SDL_KEYDOWN) {
//                 // Left in 1's
//                 if (event.key.keysym.sym == SDLK_LEFT || event.key.keysym.sym == SDLK_LSHIFT) {
//                     int newIndex = (currentIndex + tables.size() - 1) % tables.size();
//                     if (newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Left in 10's
//                 else if (event.key.keysym.sym == SDLK_DOWN || event.key.keysym.sym == SDLK_LCTRL) {
//                     int newIndex = (currentIndex + tables.size() - 10) % tables.size();
//                     if (newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Right in 1's
//                 else if (event.key.keysym.sym == SDLK_RIGHT || event.key.keysym.sym == SDLK_RSHIFT) {
//                     int newIndex = (currentIndex + 1) % tables.size();
//                     if (newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Right in 10's
//                 else if (event.key.keysym.sym == SDLK_UP || event.key.keysym.sym == SDLK_RCTRL) {
//                     int newIndex = (currentIndex + 10) % tables.size();
//                     if (newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Letter Navigation (Next)
//                 else if (event.key.keysym.sym == SDLK_SLASH) { // 'z' key
//                     char currentLetter = toupper(tables[currentIndex].tableName[0]);
//                     char nextLetter = currentLetter + 1;
//                     bool found = false;
//                     int newIndex = currentIndex;
//
//                     for (; nextLetter <= 'Z'; ++nextLetter) {
//                         if (letterIndex.find(nextLetter) != letterIndex.end()) {
//                             newIndex = letterIndex[nextLetter];
//                             found = true;
//                             break;
//                         }
//                     }
//                     if (!found) { // Wrap around
//                         for (nextLetter = 'A'; nextLetter < currentLetter; ++nextLetter) {
//                             if (letterIndex.find(nextLetter) != letterIndex.end()) {
//                                 newIndex = letterIndex[nextLetter];
//                                 found = true;
//                                 break;
//                             }
//                         }
//                     }
//                     if (found && newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Letter Navigation (Previous)
//                 else if (event.key.keysym.sym == SDLK_z) { // '/' key
//                     char currentLetter = toupper(tables[currentIndex].tableName[0]);
//                     char prevLetter = currentLetter - 1;
//                     bool found = false;
//                     int newIndex = currentIndex;
//
//                     for (; prevLetter >= 'A'; --prevLetter) {
//                         if (letterIndex.find(prevLetter) != letterIndex.end()) {
//                             newIndex = letterIndex[prevLetter];
//                             found = true;
//                             break;
//                         }
//                     }
//                     if (!found) { // Wrap around
//                         for (prevLetter = 'Z'; prevLetter > currentLetter; --prevLetter) {
//                             if (letterIndex.find(prevLetter) != letterIndex.end()) {
//                                 newIndex = letterIndex[prevLetter];
//                                 found = true;
//                                 break;
//                             }
//                         }
//                     }
//                     if (found && newIndex != currentIndex) {
//                         performTableTransition();
//                         currentIndex = newIndex;
//                     }
//                 }
//                 // Launch Table
//                 else if (event.key.keysym.sym == SDLK_RETURN || event.key.keysym.sym == SDLK_KP_ENTER) {
//                     if (tableLoadSound) Mix_PlayChannel(-1, tableLoadSound.get(), 0);
//                     launchTable(tables[currentIndex]);
//                 }
//                 // Quit
//                 else if (event.key.keysym.sym == SDLK_ESCAPE || event.key.keysym.sym == SDLK_q) {
//                     quit = true;
//                 }
//             }
//         }
//
//         // ------------------ Handle Transitions ------------------
//         Uint8 currentAlpha = 255;
//         Uint32 now = SDL_GetTicks();
//         if (transitionState != TransitionState::IDLE) {
//             Uint32 elapsed = now - transitionStartTime;
//             int halfDuration = FADE_DURATION_MS / 2;
//             if (transitionState == TransitionState::FADING_OUT) {
//                 if (elapsed < (Uint32)halfDuration) {
//                     currentAlpha = 255 - (Uint8)(((255 - FADE_TARGET_ALPHA) * elapsed) / halfDuration);
//                 } else {
//                     loadCurrentTableTextures();
//                     transitionState = TransitionState::FADING_IN;
//                     transitionStartTime = SDL_GetTicks();
//                     currentAlpha = FADE_TARGET_ALPHA;
//                 }
//             }
//             else if (transitionState == TransitionState::FADING_IN) {
//                 if (elapsed < (Uint32)halfDuration) {
//                     currentAlpha = FADE_TARGET_ALPHA + (Uint8)(((255 - FADE_TARGET_ALPHA) * elapsed) / halfDuration);
//                 } else {
//                     currentAlpha = 255;
//                     transitionState = TransitionState::IDLE;
//                 }
//             }
//         }
//
//         if (tableTexture) SDL_SetTextureAlphaMod(tableTexture, currentAlpha);
//         if (wheelTexture) SDL_SetTextureAlphaMod(wheelTexture, currentAlpha);
//         if (backglassTexture) SDL_SetTextureAlphaMod(backglassTexture, currentAlpha);
//         if (dmdTexture) SDL_SetTextureAlphaMod(dmdTexture, currentAlpha);
//         if (tableNameTexture) SDL_SetTextureAlphaMod(tableNameTexture, currentAlpha);
//         if (tableVideoCtx.texture) SDL_SetTextureAlphaMod(tableVideoCtx.texture, currentAlpha);
//         if (backglassVideoCtx.texture) SDL_SetTextureAlphaMod(backglassVideoCtx.texture, currentAlpha);
//         if (dmdVideoCtx.texture) SDL_SetTextureAlphaMod(dmdVideoCtx.texture, currentAlpha);
//
//         if (tableVideoPlayer && tableVideoCtx.texture && tableVideoCtx.updated && tableVideoCtx.pixels) {
//             SDL_LockMutex(tableVideoCtx.mutex);
//             SDL_UpdateTexture(tableVideoCtx.texture, nullptr, tableVideoCtx.pixels, tableVideoCtx.pitch);
//             tableVideoCtx.updated = false;
//             SDL_UnlockMutex(tableVideoCtx.mutex);
//         }
//
//         if (backglassVideoPlayer && backglassVideoCtx.texture && backglassVideoCtx.updated && backglassVideoCtx.pixels) {
//             SDL_LockMutex(backglassVideoCtx.mutex);
//             SDL_UpdateTexture(backglassVideoCtx.texture, nullptr, backglassVideoCtx.pixels, backglassVideoCtx.pitch);
//             backglassVideoCtx.updated = false;
//             SDL_UnlockMutex(backglassVideoCtx.mutex);
//         }
//
//         if (dmdVideoPlayer && dmdVideoCtx.texture && dmdVideoCtx.updated && dmdVideoCtx.pixels) {
//             SDL_LockMutex(dmdVideoCtx.mutex);
//             SDL_UpdateTexture(dmdVideoCtx.texture, nullptr, dmdVideoCtx.pixels, dmdVideoCtx.pitch);
//             dmdVideoCtx.updated = false;
//             SDL_UnlockMutex(dmdVideoCtx.mutex);
//         }
//
//         // ------------------------------------------------------------------------
//         // -------------------------- Render Primary Screen -----------------------
//         SDL_SetRenderDrawColor(primaryRenderer.get(), 32, 32, 32, 255);
//         SDL_RenderClear(primaryRenderer.get());
//
//         SDL_Rect tableRect = {0, 0, MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT};
//         if (tableVideoPlayer && tableVideoCtx.texture) {
//             SDL_RenderCopy(primaryRenderer.get(), tableVideoCtx.texture, nullptr, &tableRect);
//         } else if (tableTexture) {
//             SDL_RenderCopy(primaryRenderer.get(), tableTexture, nullptr, &tableRect);
//         }
//
//         if (wheelTexture) {
//             SDL_Rect wheelRect;
//             wheelRect.w = WHEEL_IMAGE_SIZE;
//             wheelRect.h = WHEEL_IMAGE_SIZE;
//             wheelRect.x = MAIN_WINDOW_WIDTH - WHEEL_IMAGE_SIZE - WHEEL_IMAGE_MARGIN;
//             wheelRect.y = MAIN_WINDOW_HEIGHT - WHEEL_IMAGE_SIZE - WHEEL_IMAGE_MARGIN;
//             SDL_RenderCopy(primaryRenderer.get(), wheelTexture, nullptr, &wheelRect);
//         }
//
//         if (tableNameTexture) {
//             SDL_Rect backgroundRect = {tableNameRect.x - 5, tableNameRect.y - 5, tableNameRect.w + 10, tableNameRect.h + 10};
//             SDL_SetRenderDrawColor(primaryRenderer.get(), 0, 0, 0, 128);
//             SDL_RenderFillRect(primaryRenderer.get(), &backgroundRect);
//             SDL_RenderCopy(primaryRenderer.get(), tableNameTexture, nullptr, &tableNameRect);
//         }
//
//         SDL_RenderPresent(primaryRenderer.get());
//
//         // ------------------------------------------------------------------------
//         // ----------------------- Render Secondary Screen ------------------------
//         SDL_SetRenderDrawColor(secondaryRenderer.get(), 0, 0, 0, 255);
//         SDL_RenderClear(secondaryRenderer.get());
//
//         SDL_Rect backglassRect = {0, 0, BACKGLASS_MEDIA_WIDTH, BACKGLASS_MEDIA_HEIGHT};
//         if (backglassVideoPlayer && backglassVideoCtx.texture) {
//             SDL_RenderCopy(secondaryRenderer.get(), backglassVideoCtx.texture, nullptr, &backglassRect);
//         } else if (backglassTexture) {
//             SDL_RenderCopy(secondaryRenderer.get(), backglassTexture, nullptr, &backglassRect);
//         }
//
//         SDL_Rect dmdRect = {0, BACKGLASS_MEDIA_HEIGHT, DMD_MEDIA_WIDTH, DMD_MEDIA_HEIGHT};
//         if (dmdVideoPlayer && dmdVideoCtx.texture) {
//             SDL_RenderCopy(secondaryRenderer.get(), dmdVideoCtx.texture, nullptr, &dmdRect);
//         } else if (dmdTexture) {
//             SDL_RenderCopy(secondaryRenderer.get(), dmdTexture, nullptr, &dmdRect);
//         }
//
//         SDL_RenderPresent(secondaryRenderer.get());
//
//         SDL_Delay(16);
//     }
//
//     // ----------------- Cleanup ------------------------
//     // Only clean up resources not managed by unique_ptr
//     cleanupVideoContext(tableVideoCtx, tableVideoPlayer);
//     cleanupVideoContext(backglassVideoCtx, backglassVideoPlayer);
//     cleanupVideoContext(dmdVideoCtx, dmdVideoPlayer);
//
//     if (tableTexture) SDL_DestroyTexture(tableTexture);
//     if (wheelTexture) SDL_DestroyTexture(wheelTexture);
//     if (backglassTexture) SDL_DestroyTexture(backglassTexture);
//     if (dmdTexture) SDL_DestroyTexture(dmdTexture);
//     if (tableNameTexture) SDL_DestroyTexture(tableNameTexture);
//
//     // No need to manually destroy font, sounds, renderers, windows, or call library quit functions
//     // (handled by unique_ptr and guard classes)
//     return 0;
// }

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    run().unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        ExitCode::FAILURE
    })
}

fn run() -> Result<ExitCode, sdl3::Error> {
    let sdl_context = sdl3::init()?;
    let sdl_video = sdl_context.video()?;
    let sdl_audio = sdl_context.audio()?;

    let window = sdl_video
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(ExitCode::SUCCESS)
}
