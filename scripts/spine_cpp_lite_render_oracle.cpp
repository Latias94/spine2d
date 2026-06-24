#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <vector>

#include "spine-c.h"

static std::string read_file(const char *path) {
  std::ifstream file(path, std::ios::binary);
  if (!file) {
    std::cerr << "failed to open: " << path << "\n";
    std::exit(2);
  }
  std::ostringstream ss;
  ss << file.rdbuf();
  return ss.str();
}

static void usage() {
  std::cerr
      << "Usage:\n"
         "  spine_cpp_lite_render_oracle <atlas.atlas> <skeleton.(json|skel)> [--y-down 0|1] <commands...>\n"
         "\n"
         "Commands:\n"
         "  --set-skin <name|none>\n"
         "  --physics <none|reset|update|pose>\n"
         "  --mix <from> <to> <duration>\n"
         "  --set <track> <animation> <loop 0|1>\n"
         "  --add <track> <animation> <loop 0|1> <delay>\n"
         "  --set-empty <track> <mixDuration>\n"
         "  --add-empty <track> <mixDuration> <delay>\n"
         "  --entry-alpha <alpha>\n"
         "  --entry-event-threshold <threshold>\n"
         "  --entry-alpha-attachment-threshold <threshold>\n"
         "  --entry-mix-attachment-threshold <threshold>\n"
         "  --entry-mix-draw-order-threshold <threshold>\n"
         "  --entry-additive <0|1>\n"
         "  --entry-mix-interpolation <linear|smooth|slow-fast|fast-slow|circle>\n"
         "  --entry-reverse <0|1>\n"
         "  --entry-shortest-rotation <0|1>\n"
         "  --entry-reset-rotation-directions\n"
         "  --step <dt>\n";
}

static std::string json_escape(const char *s) {
  if (!s) return "";
  std::string out;
  for (const char *p = s; *p; p++) {
    const unsigned char c = static_cast<unsigned char>(*p);
    if (c == '\\') out += "\\\\";
    else if (c == '\"') out += "\\\"";
    else if (c == '\n') out += "\\n";
    else if (c == '\r') out += "\\r";
    else if (c == '\t') out += "\\t";
    else out += static_cast<char>(c);
  }
  return out;
}

static bool parse_mix_interpolation(const char *s, spine_interpolation *out) {
  if (std::strcmp(s, "linear") == 0) {
    *out = spine_interpolation_linear();
    return true;
  }
  if (std::strcmp(s, "smooth") == 0) {
    *out = spine_interpolation_smooth();
    return true;
  }
  if (std::strcmp(s, "slow-fast") == 0) {
    *out = spine_interpolation_slow_fast();
    return true;
  }
  if (std::strcmp(s, "fast-slow") == 0) {
    *out = spine_interpolation_fast_slow();
    return true;
  }
  if (std::strcmp(s, "circle") == 0) {
    *out = spine_interpolation_circle();
    return true;
  }
  return false;
}

static const char *blend_mode_name(spine_blend_mode mode) {
  switch (mode) {
    case SPINE_BLEND_MODE_NORMAL: return "normal";
    case SPINE_BLEND_MODE_ADDITIVE: return "additive";
    case SPINE_BLEND_MODE_MULTIPLY: return "multiply";
    case SPINE_BLEND_MODE_SCREEN: return "screen";
    default: return "unknown";
  }
}

static const char *physics_name(spine_physics physics) {
  switch (physics) {
    case SPINE_PHYSICS_NONE: return "none";
    case SPINE_PHYSICS_RESET: return "reset";
    case SPINE_PHYSICS_UPDATE: return "update";
    case SPINE_PHYSICS_POSE: return "pose";
    default: return "unknown";
  }
}

static uint32_t premultiply_packed_aarrggbb(uint32_t c) {
  const uint8_t a8 = static_cast<uint8_t>(c >> 24);
  const float a = static_cast<float>(a8) / 255.0f;
  const uint8_t r8 = static_cast<uint8_t>(((c >> 16) & 0xff) * a);
  const uint8_t g8 = static_cast<uint8_t>(((c >> 8) & 0xff) * a);
  const uint8_t b8 = static_cast<uint8_t>(((c >> 0) & 0xff) * a);
  return (static_cast<uint32_t>(a8) << 24) | (static_cast<uint32_t>(r8) << 16) |
         (static_cast<uint32_t>(g8) << 8) | static_cast<uint32_t>(b8);
}

static uint32_t adjust_dark_color_for_shader(uint32_t dark, uint32_t light, bool premultipliedAlpha) {
  const uint32_t rgb = dark & 0x00ffffffu;

  // No dark color: keep (0,0,0,1) which makes the shader a no-op for the dark term.
  if (rgb == 0) {
    return 0xff000000u;
  }

  uint8_t r8 = static_cast<uint8_t>((dark >> 16) & 0xff);
  uint8_t g8 = static_cast<uint8_t>((dark >> 8) & 0xff);
  uint8_t b8 = static_cast<uint8_t>((dark >> 0) & 0xff);

  // `spine-ts/spine-webgl` uses darkColor.a as a PMA switch:
  // - PMA: dark.rgb premultiplied by final alpha, dark.a=1
  // - non-PMA: dark.rgb not premultiplied, dark.a=0
  if (premultipliedAlpha) {
    const uint8_t a8 = static_cast<uint8_t>(light >> 24);
    const float a = static_cast<float>(a8) / 255.0f;
    r8 = static_cast<uint8_t>(r8 * a);
    g8 = static_cast<uint8_t>(g8 * a);
    b8 = static_cast<uint8_t>(b8 * a);
    return (0xffu << 24) | (static_cast<uint32_t>(r8) << 16) | (static_cast<uint32_t>(g8) << 8) |
           static_cast<uint32_t>(b8);
  }

  return (0x00u << 24) | (static_cast<uint32_t>(r8) << 16) | (static_cast<uint32_t>(g8) << 8) |
         static_cast<uint32_t>(b8);
}

int main(int argc, char **argv) {
  if (argc < 4) {
    usage();
    return 2;
  }

  // Ensure float values round-trip when parsed by JSON readers.
  std::cout << std::setprecision(std::numeric_limits<float>::max_digits10);

  const char *atlas_path = argv[1];
  const char *skeleton_path = argv[2];

  const char *skin = nullptr;
  const char *anim = "<scenario>";
  float time = 0.0f;
  int y_down = 0;
  spine_physics physics = SPINE_PHYSICS_NONE;

  // Parse global options first. Scenario commands are parsed later.
  for (int i = 3; i < argc; i++) {
    const char *arg = argv[i];
    if (std::strcmp(arg, "--y-down") == 0 && i + 1 < argc) {
      y_down = std::atoi(argv[++i]) ? 1 : 0;
    }
  }

  spine_bone_set_y_down(y_down ? true : false);

  std::string atlas_text = read_file(atlas_path);
  spine_atlas_result atlas_result = spine_atlas_load(atlas_text.c_str());
  if (!atlas_result) {
    std::cerr << "spine_atlas_load failed\n";
    return 2;
  }
  const char *atlas_err = spine_atlas_result_get_error(atlas_result);
  if (atlas_err && atlas_err[0]) {
    std::cerr << "atlas error: " << atlas_err << "\n";
    return 2;
  }
  spine_atlas atlas = spine_atlas_result_get_atlas(atlas_result);
  if (!atlas) {
    std::cerr << "missing atlas\n";
    return 2;
  }

  spine_skeleton_data_result data_result = nullptr;
  const std::string skeleton_path_s(skeleton_path);
  if (skeleton_path_s.size() >= 5 &&
      skeleton_path_s.compare(skeleton_path_s.size() - 5, 5, ".skel") == 0) {
    std::string bytes = read_file(skeleton_path);
    data_result = spine_skeleton_data_load_binary(
        atlas, reinterpret_cast<const uint8_t *>(bytes.data()), (int32_t)bytes.size(), skeleton_path);
  } else {
    std::string json_text = read_file(skeleton_path);
    data_result =
        spine_skeleton_data_load_json(atlas, json_text.c_str(), skeleton_path);
  }
  if (!data_result) {
    std::cerr << "spine_skeleton_data_load_(json|binary) failed\n";
    return 2;
  }
  const char *skeleton_err = spine_skeleton_data_result_get_error(data_result);
  if (skeleton_err && skeleton_err[0]) {
    std::cerr << "skeleton data error: " << skeleton_err << "\n";
    return 2;
  }
  spine_skeleton_data data = spine_skeleton_data_result_get_data(data_result);
  if (!data) {
    std::cerr << "missing skeleton data\n";
    return 2;
  }

  spine_bone_set_y_down(y_down ? true : false);

  spine_skeleton_drawable drawable = spine_skeleton_drawable_create(data);
  if (!drawable) {
    std::cerr << "spine_skeleton_drawable_create failed\n";
    return 2;
  }

  spine_skeleton skeleton = spine_skeleton_drawable_get_skeleton(drawable);
  spine_animation_state state = spine_skeleton_drawable_get_animation_state(drawable);
  spine_animation_state_data state_data = spine_skeleton_drawable_get_animation_state_data(drawable);
  if (!skeleton || !state || !state_data) {
    std::cerr << "missing skeleton/state/state_data\n";
    return 2;
  }

  float total_time = 0.0f;
  spine_track_entry last_entry = nullptr;

  spine_skeleton_setup_pose(skeleton);

  if (skin) {
    if (std::strcmp(skin, "none") == 0) {
      spine_skeleton_set_skin_2(skeleton, nullptr);
    } else {
      spine_skeleton_set_skin_1(skeleton, skin);
    }
    spine_skeleton_setup_pose_slots(skeleton);
    spine_skeleton_update_cache(skeleton);
  }

  for (int i = 3; i < argc; i++) {
    const char *arg = argv[i];

    if (std::strcmp(arg, "--y-down") == 0) {
      i++;  // already processed above
      continue;
    }

    if (std::strcmp(arg, "--set-skin") == 0 && i + 1 < argc) {
      const char *name = argv[i + 1];
      if (std::strcmp(name, "none") == 0) skin = nullptr;
      else skin = name;
      if (skin) {
        spine_skeleton_set_skin_1(skeleton, skin);
      } else {
        spine_skeleton_set_skin_2(skeleton, nullptr);
      }
      spine_skeleton_setup_pose_slots(skeleton);
      spine_skeleton_update_cache(skeleton);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--mix") == 0 && i + 3 < argc) {
      const char *from_name = argv[i + 1];
      const char *to_name = argv[i + 2];
      const float duration = std::strtof(argv[i + 3], nullptr);
      spine_animation_state_data_set_mix_1(state_data, from_name, to_name, duration);
      i += 3;
      continue;
    }

    if (std::strcmp(arg, "--physics") == 0 && i + 1 < argc) {
      const char *mode = argv[i + 1];
      if (std::strcmp(mode, "none") == 0) physics = SPINE_PHYSICS_NONE;
      else if (std::strcmp(mode, "reset") == 0) physics = SPINE_PHYSICS_RESET;
      else if (std::strcmp(mode, "update") == 0) physics = SPINE_PHYSICS_UPDATE;
      else if (std::strcmp(mode, "pose") == 0) physics = SPINE_PHYSICS_POSE;
      else {
        std::cerr << "invalid physics mode: " << mode << "\n";
        return 2;
      }
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--set") == 0 && i + 3 < argc) {
      const size_t track = (size_t)std::atoi(argv[i + 1]);
      const char *name = argv[i + 2];
      const bool looped = std::atoi(argv[i + 3]) ? true : false;
      last_entry = spine_animation_state_set_animation_1(state, track, name, looped);
      i += 3;
      continue;
    }

    if (std::strcmp(arg, "--add") == 0 && i + 4 < argc) {
      const size_t track = (size_t)std::atoi(argv[i + 1]);
      const char *name = argv[i + 2];
      const bool looped = std::atoi(argv[i + 3]) ? true : false;
      const float delay = std::strtof(argv[i + 4], nullptr);
      last_entry = spine_animation_state_add_animation_1(state, track, name, looped, delay);
      i += 4;
      continue;
    }

    if (std::strcmp(arg, "--set-empty") == 0 && i + 2 < argc) {
      const size_t track = (size_t)std::atoi(argv[i + 1]);
      const float mix = std::strtof(argv[i + 2], nullptr);
      last_entry = spine_animation_state_set_empty_animation(state, track, mix);
      i += 2;
      continue;
    }

    if (std::strcmp(arg, "--add-empty") == 0 && i + 3 < argc) {
      const size_t track = (size_t)std::atoi(argv[i + 1]);
      const float mix = std::strtof(argv[i + 2], nullptr);
      const float delay = std::strtof(argv[i + 3], nullptr);
      last_entry = spine_animation_state_add_empty_animation(state, track, mix, delay);
      i += 3;
      continue;
    }

    if (std::strcmp(arg, "--entry-alpha") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-alpha requires a preceding --set/--add command\n";
        return 2;
      }
      const float alpha = std::strtof(argv[i + 1], nullptr);
      spine_track_entry_set_alpha(last_entry, alpha);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-event-threshold") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-event-threshold requires a preceding --set/--add command\n";
        return 2;
      }
      const float threshold = std::strtof(argv[i + 1], nullptr);
      spine_track_entry_set_event_threshold(last_entry, threshold);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-alpha-attachment-threshold") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-alpha-attachment-threshold requires a preceding --set/--add command\n";
        return 2;
      }
      const float threshold = std::strtof(argv[i + 1], nullptr);
      spine_track_entry_set_alpha_attachment_threshold(last_entry, threshold);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-mix-attachment-threshold") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-mix-attachment-threshold requires a preceding --set/--add command\n";
        return 2;
      }
      const float threshold = std::strtof(argv[i + 1], nullptr);
      spine_track_entry_set_mix_attachment_threshold(last_entry, threshold);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-mix-draw-order-threshold") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-mix-draw-order-threshold requires a preceding --set/--add command\n";
        return 2;
      }
      const float threshold = std::strtof(argv[i + 1], nullptr);
      spine_track_entry_set_mix_draw_order_threshold(last_entry, threshold);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-additive") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-additive requires a preceding --set/--add command\n";
        return 2;
      }
      const bool additive = std::atoi(argv[i + 1]) ? true : false;
      spine_track_entry_set_additive(last_entry, additive);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-mix-interpolation") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-mix-interpolation requires a preceding --set/--add command\n";
        return 2;
      }
      spine_interpolation interpolation;
      if (!parse_mix_interpolation(argv[i + 1], &interpolation)) {
        std::cerr << "--entry-mix-interpolation must be linear|smooth|slow-fast|fast-slow|circle\n";
        return 2;
      }
      spine_track_entry_set_mix_interpolation(last_entry, interpolation);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-reverse") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-reverse requires a preceding --set/--add command\n";
        return 2;
      }
      const bool reverse = std::atoi(argv[i + 1]) ? true : false;
      spine_track_entry_set_reverse(last_entry, reverse);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-shortest-rotation") == 0 && i + 1 < argc) {
      if (!last_entry) {
        std::cerr << "--entry-shortest-rotation requires a preceding --set/--add command\n";
        return 2;
      }
      const bool shortest = std::atoi(argv[i + 1]) ? true : false;
      spine_track_entry_set_shortest_rotation(last_entry, shortest);
      i += 1;
      continue;
    }

    if (std::strcmp(arg, "--entry-reset-rotation-directions") == 0) {
      if (!last_entry) {
        std::cerr << "--entry-reset-rotation-directions requires a preceding --set/--add command\n";
        return 2;
      }
      spine_track_entry_reset_rotation_directions(last_entry);
      continue;
    }

    if (std::strcmp(arg, "--step") == 0 && i + 1 < argc) {
      const float dt = std::strtof(argv[i + 1], nullptr);
      spine_animation_state_update(state, dt);
      spine_animation_state_apply(state, skeleton);
      spine_skeleton_update(skeleton, dt);
      spine_skeleton_update_world_transform(skeleton, physics);
      total_time += dt;
      i += 1;
      continue;
    }

    std::cerr << "unknown/invalid command: " << arg << "\n";
    usage();
    return 2;
  }

  anim = "<scenario>";
  time = total_time;

  spine_render_command cmd = spine_skeleton_drawable_render(drawable);

  bool premultipliedAlpha = false;
  {
    spine_array_atlas_page pages = spine_atlas_get_pages(atlas);
    const size_t n = spine_array_atlas_page_size(pages);
    spine_atlas_page *buf = spine_array_atlas_page_buffer(pages);
    for (size_t i = 0; i < n; i++) {
      if (spine_atlas_page_get_pma(buf[i])) {
        premultipliedAlpha = true;
        break;
      }
    }
  }

  std::cout << "{";
  std::cout << "\"mode\":\"scenario\",";
  std::cout << "\"y_down\":" << y_down << ",";
  std::cout << "\"pma\":" << (premultipliedAlpha ? 1 : 0) << ",";
  std::cout << "\"physics\":\"" << physics_name(physics) << "\",";
  std::cout << "\"skin\":" << (skin ? ("\"" + json_escape(skin) + "\"") : "null") << ",";
  std::cout << "\"anim\":\"" << json_escape(anim) << "\",";
  std::cout << "\"time\":" << time << ",";
  std::cout << "\"draws\":[";

  bool first_cmd = true;
  while (cmd) {
    const int32_t page = (int32_t)(intptr_t)spine_render_command_get_texture(cmd);
    const spine_blend_mode blend = spine_render_command_get_blend_mode(cmd);
    const int32_t num_vertices = spine_render_command_get_num_vertices(cmd);
    const int32_t num_indices = spine_render_command_get_num_indices(cmd);

    float *positions = spine_render_command_get_positions(cmd);
    float *uvs = spine_render_command_get_uvs(cmd);
    uint32_t *colors = spine_render_command_get_colors(cmd);
    uint32_t *dark_colors = spine_render_command_get_dark_colors(cmd);
    uint16_t *indices = spine_render_command_get_indices(cmd);

    if (!first_cmd) std::cout << ",";
    first_cmd = false;

    std::cout << "{";
    std::cout << "\"page\":" << page << ",";
    std::cout << "\"blend\":\"" << blend_mode_name(blend) << "\",";
    std::cout << "\"num_vertices\":" << num_vertices << ",";
    std::cout << "\"num_indices\":" << num_indices << ",";

    std::cout << "\"positions\":[";
    for (int32_t i = 0; i < num_vertices * 2; i++) {
      if (i) std::cout << ",";
      std::cout << positions[i];
    }
    std::cout << "],";

    std::cout << "\"uvs\":[";
    for (int32_t i = 0; i < num_vertices * 2; i++) {
      if (i) std::cout << ",";
      std::cout << uvs[i];
    }
    std::cout << "],";

    std::cout << "\"colors\":[";
    for (int32_t i = 0; i < num_vertices; i++) {
      if (i) std::cout << ",";
      uint32_t c = (uint32_t)colors[i];
      if (premultipliedAlpha) c = premultiply_packed_aarrggbb(c);
      std::cout << c;
    }
    std::cout << "],";

    std::cout << "\"dark_colors\":[";
    for (int32_t i = 0; i < num_vertices; i++) {
      if (i) std::cout << ",";
      uint32_t light = (uint32_t)colors[i];
      if (premultipliedAlpha) light = premultiply_packed_aarrggbb(light);
      uint32_t dark = (uint32_t)dark_colors[i];
      dark = adjust_dark_color_for_shader(dark, (uint32_t)colors[i], premultipliedAlpha);
      std::cout << dark;
    }
    std::cout << "],";

    std::cout << "\"indices\":[";
    for (int32_t i = 0; i < num_indices; i++) {
      if (i) std::cout << ",";
      std::cout << indices[i];
    }
    std::cout << "]";

    std::cout << "}";

    cmd = spine_render_command_get_next(cmd);
  }

  std::cout << "]}";
  std::cout << "\n";

  spine_skeleton_drawable_dispose(drawable);
  spine_skeleton_data_result_dispose(data_result);
  spine_atlas_dispose(atlas);
  spine_atlas_result_dispose(atlas_result);

  return 0;
}
