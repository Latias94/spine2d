#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cctype>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

#include "spine-c.h"

// PhysicsConstraint runtime state fields are private in spine-cpp. For oracle/debugging, we
// temporarily widen access to compare internal state to our Rust implementation.
#define private public
#include <spine/PhysicsConstraint.h>
#undef private

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
         "  spine_cpp_lite_oracle <atlas.atlas> <skeleton.(json|skel)> <animation> <time> [--y-down 0|1] [--physics none|reset|update|pose]\n"
         "\n"
         "Scenario mode:\n"
         "  spine_cpp_lite_oracle <atlas.atlas> <skeleton.(json|skel)> [--y-down 0|1] [--physics none|reset|update|pose] <commands...>\n"
         "\n"
         "Commands (scenario mode):\n"
         "  --set-skin <name|none>\n"
         "  --physics <none|reset|update|pose>\n"
         "  --mix <from> <to> <duration>\n"
         "  --set <track> <animation> <loop 0|1>\n"
         "  --add <track> <animation> <loop 0|1> <delay>\n"
         "  --set-empty <track> <mixDuration>\n"
         "  --add-empty <track> <mixDuration> <delay>\n"
         "  --dump-slot-vertices <slotName>\n"
         "  --entry-alpha <alpha>\n"
         "  --entry-event-threshold <threshold>\n"
         "  --entry-alpha-attachment-threshold <threshold>\n"
         "  --entry-mix-attachment-threshold <threshold>\n"
         "  --entry-mix-draw-order-threshold <threshold>\n"
         "  --entry-mix-blend <setup|first|replace|add>\n"
         "  --entry-reverse <0|1>\n"
         "  --entry-shortest-rotation <0|1>\n"
         "  --entry-reset-rotation-directions\n"
         "  --dump-update-cache\n"
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

static bool debug_bone_name_enabled(const char *name, const char *filter) {
  if (!name || !filter || !filter[0]) return false;
  const std::string needle(name);
  std::string current;
  for (const char *p = filter;; ++p) {
    if (*p == ',' || *p == '\0') {
      size_t start = 0;
      while (start < current.size() && std::isspace(static_cast<unsigned char>(current[start]))) start++;
      size_t end = current.size();
      while (end > start && std::isspace(static_cast<unsigned char>(current[end - 1]))) end--;
      if (current.substr(start, end - start) == needle) return true;
      current.clear();
      if (*p == '\0') return false;
    } else {
      current.push_back(*p);
    }
  }
}

static bool parse_mix_blend(const char *s, spine_mix_blend *out) {
  if (std::strcmp(s, "setup") == 0) {
    *out = SPINE_MIX_BLEND_SETUP;
    return true;
  }
  if (std::strcmp(s, "first") == 0) {
    *out = SPINE_MIX_BLEND_FIRST;
    return true;
  }
  if (std::strcmp(s, "replace") == 0) {
    *out = SPINE_MIX_BLEND_REPLACE;
    return true;
  }
  if (std::strcmp(s, "add") == 0) {
    *out = SPINE_MIX_BLEND_ADD;
    return true;
  }
  return false;
}

static void debug_dump_bones(const char *label, spine_skeleton skeleton, float total_time) {
  const char *filter = std::getenv("SPINE2D_DEBUG_BONES");
  if (!filter || !filter[0]) return;

  spine_array_bone bones = spine_skeleton_get_bones(skeleton);
  const size_t nb = spine_array_bone_size(bones);
  spine_bone *bones_buf = spine_array_bone_buffer(bones);
  std::cerr << std::setprecision(std::numeric_limits<float>::max_digits10);
  std::cerr << "[DEBUG-runwalk] " << label << " t=" << total_time << "\n";
  for (size_t i = 0; i < nb; i++) {
    spine_bone bone = bones_buf[i];
    spine_bone_data bd = spine_bone_get_data(bone);
    const char *name = bd ? spine_bone_data_get_name(bd) : "<unknown>";
    if (!debug_bone_name_enabled(name, filter)) continue;
    spine_bone_local pose = spine_bone_get_pose(bone);
    spine_bone_pose applied = spine_bone_get_applied_pose(bone);
    std::cerr << "[DEBUG-runwalk] bone[" << i << "] " << name
              << " pose x=" << spine_bone_local_get_x(pose)
              << " y=" << spine_bone_local_get_y(pose)
              << " rot=" << spine_bone_local_get_rotation(pose)
              << " sx=" << spine_bone_local_get_scale_x(pose)
              << " sy=" << spine_bone_local_get_scale_y(pose)
              << " shx=" << spine_bone_local_get_shear_x(pose)
              << " shy=" << spine_bone_local_get_shear_y(pose)
              << " world a=" << spine_bone_pose_get_a(applied)
              << " b=" << spine_bone_pose_get_b(applied)
              << " c=" << spine_bone_pose_get_c(applied)
              << " d=" << spine_bone_pose_get_d(applied)
              << " x=" << spine_bone_pose_get_world_x(applied)
              << " y=" << spine_bone_pose_get_world_y(applied)
              << " applied x=" << spine_bone_pose_get_x(applied)
              << " y=" << spine_bone_pose_get_y(applied)
              << " rot=" << spine_bone_pose_get_rotation(applied)
              << " sx=" << spine_bone_pose_get_scale_x(applied)
              << " sy=" << spine_bone_pose_get_scale_y(applied)
              << " shx=" << spine_bone_pose_get_shear_x(applied)
              << " shy=" << spine_bone_pose_get_shear_y(applied)
              << " world a=" << spine_bone_pose_get_a(applied)
              << " b=" << spine_bone_pose_get_b(applied)
              << " c=" << spine_bone_pose_get_c(applied)
              << " d=" << spine_bone_pose_get_d(applied)
              << " x=" << spine_bone_pose_get_world_x(applied)
              << " y=" << spine_bone_pose_get_world_y(applied)
              << "\n";
  }
}

struct AttachmentTypeInfo {
  int type;
  const char *name;
};

static AttachmentTypeInfo attachment_type_info(spine_attachment att) {
  if (!att) return {-1, "unknown"};
  const spine_rtti r = spine_attachment_get_rtti(att);
  if (spine_rtti_instance_of(r, spine_region_attachment_rtti())) return {0, "region"};
  if (spine_rtti_instance_of(r, spine_mesh_attachment_rtti())) return {1, "mesh"};
  if (spine_rtti_instance_of(r, spine_clipping_attachment_rtti())) return {2, "clipping"};
  if (spine_rtti_instance_of(r, spine_bounding_box_attachment_rtti())) return {3, "boundingbox"};
  if (spine_rtti_instance_of(r, spine_path_attachment_rtti())) return {4, "path"};
  if (spine_rtti_instance_of(r, spine_point_attachment_rtti())) return {5, "point"};
  return {-1, "unknown"};
}

static spine_atlas load_atlas_or_die(const char *atlas_path, spine_atlas_result &out_result) {
  std::string atlas_text = read_file(atlas_path);
  out_result = spine_atlas_load(atlas_text.c_str());
  if (!out_result) {
    std::cerr << "spine_atlas_load failed\n";
    std::exit(2);
  }
  const char *err = spine_atlas_result_get_error(out_result);
  if (err && err[0]) {
    std::cerr << "atlas error: " << err << "\n";
    std::exit(2);
  }
  spine_atlas atlas = spine_atlas_result_get_atlas(out_result);
  if (!atlas) {
    std::cerr << "missing atlas\n";
    std::exit(2);
  }
  return atlas;
}

static spine_skeleton_data load_skeleton_data_or_die(
    spine_atlas atlas,
    const char *skeleton_path,
    spine_skeleton_data_result &out_result) {
  out_result = nullptr;
  const std::string path_s(skeleton_path);
  if (path_s.size() >= 5 && path_s.compare(path_s.size() - 5, 5, ".skel") == 0) {
    std::string bytes = read_file(skeleton_path);
    out_result = spine_skeleton_data_load_binary(
        atlas, reinterpret_cast<const uint8_t *>(bytes.data()), (int32_t)bytes.size(), skeleton_path);
  } else {
    std::string json_text = read_file(skeleton_path);
    out_result = spine_skeleton_data_load_json(atlas, json_text.c_str(), skeleton_path);
  }

  if (!out_result) {
    std::cerr << "spine_skeleton_data_load_(json|binary) failed\n";
    std::exit(2);
  }
  const char *err = spine_skeleton_data_result_get_error(out_result);
  if (err && err[0]) {
    std::cerr << "skeleton data error: " << err << "\n";
    std::exit(2);
  }
  spine_skeleton_data data = spine_skeleton_data_result_get_data(out_result);
  if (!data) {
    std::cerr << "missing skeleton data\n";
    std::exit(2);
  }
  return data;
}

static bool ends_with(const char *s, const char *suffix) {
  const size_t n = std::strlen(suffix);
  const size_t m = std::strlen(s);
  if (m < n) return false;
  return std::strcmp(s + (m - n), suffix) == 0;
}

int main(int argc, char **argv) {
  if (argc < 3) {
    usage();
    return 2;
  }

  std::cout << std::setprecision(std::numeric_limits<float>::max_digits10);

  const char *atlas_path = argv[1];
  const char *skeleton_path = argv[2];

  bool legacy_mode = false;
  const char *animation = "";
  float time = 0.0f;
  if (argc >= 5 && argv[3][0] != '-') {
    legacy_mode = true;
    animation = argv[3];
    time = std::strtof(argv[4], nullptr);
  }

  int y_down = 0;
  spine_physics physics = SPINE_PHYSICS_NONE;
  const char *dump_slot_vertices = nullptr;
  bool dump_update_cache = false;
  const int arg_start = legacy_mode ? 5 : 3;
  for (int i = arg_start; i < argc; i++) {
    if (std::strcmp(argv[i], "--y-down") == 0 && i + 1 < argc) {
      y_down = std::atoi(argv[i + 1]) ? 1 : 0;
      i++;
      continue;
    }
    if (legacy_mode && std::strcmp(argv[i], "--physics") == 0 && i + 1 < argc) {
      const char *mode = argv[i + 1];
      if (std::strcmp(mode, "none") == 0) physics = SPINE_PHYSICS_NONE;
      else if (std::strcmp(mode, "reset") == 0) physics = SPINE_PHYSICS_RESET;
      else if (std::strcmp(mode, "update") == 0) physics = SPINE_PHYSICS_UPDATE;
      else if (std::strcmp(mode, "pose") == 0) physics = SPINE_PHYSICS_POSE;
      else {
        std::cerr << "invalid physics mode: " << mode << "\n";
        return 2;
      }
      i++;
      continue;
    }
    if (legacy_mode && std::strcmp(argv[i], "--dump-slot-vertices") == 0 && i + 1 < argc) {
      dump_slot_vertices = argv[i + 1];
      i++;
      continue;
    }
    if (std::strcmp(argv[i], "--dump-update-cache") == 0) {
      dump_update_cache = true;
      continue;
    }
  }

  spine_bone_set_y_down(y_down ? true : false);

  spine_atlas_result atlas_result = nullptr;
  spine_atlas atlas = load_atlas_or_die(atlas_path, atlas_result);

  spine_skeleton_data_result data_result = nullptr;
  spine_skeleton_data data = load_skeleton_data_or_die(atlas, skeleton_path, data_result);

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

  if (legacy_mode) {
    spine_animation_state_set_animation_1(state, 0, animation, true);
    spine_animation_state_update(state, time);
    spine_animation_state_apply(state, skeleton);
    spine_skeleton_update(skeleton, time);
    spine_skeleton_update_world_transform(skeleton, physics);
    total_time = time;
  } else {
    for (int i = 3; i < argc; i++) {
      const char *arg = argv[i];

      if (std::strcmp(arg, "--y-down") == 0) {
        i++;  // already processed above
        continue;
      }

      if (std::strcmp(arg, "--set-skin") == 0 && i + 1 < argc) {
        const char *name = argv[i + 1];
        if (std::strcmp(name, "none") == 0) spine_skeleton_set_skin_2(skeleton, nullptr);
        else spine_skeleton_set_skin_1(skeleton, name);
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

      if (std::strcmp(arg, "--dump-slot-vertices") == 0 && i + 1 < argc) {
        dump_slot_vertices = argv[i + 1];
        i += 1;
        continue;
      }

      if (std::strcmp(arg, "--dump-update-cache") == 0) {
        dump_update_cache = true;
        continue;
      }

      if (std::strcmp(arg, "--set") == 0 && i + 3 < argc) {
        const size_t track = (size_t)std::atoi(argv[i + 1]);
        const char *name = argv[i + 2];
        const bool loop = std::atoi(argv[i + 3]) ? true : false;
        last_entry = spine_animation_state_set_animation_1(state, track, name, loop);
        i += 3;
        continue;
      }

      if (std::strcmp(arg, "--add") == 0 && i + 4 < argc) {
        const size_t track = (size_t)std::atoi(argv[i + 1]);
        const char *name = argv[i + 2];
        const bool loop = std::atoi(argv[i + 3]) ? true : false;
        const float delay = std::strtof(argv[i + 4], nullptr);
        last_entry = spine_animation_state_add_animation_1(state, track, name, loop, delay);
        i += 4;
        continue;
      }

      if (std::strcmp(arg, "--set-empty") == 0 && i + 2 < argc) {
        const size_t track = (size_t)std::atoi(argv[i + 1]);
        const float mix_duration = std::strtof(argv[i + 2], nullptr);
        last_entry = spine_animation_state_set_empty_animation(state, track, mix_duration);
        i += 2;
        continue;
      }

      if (std::strcmp(arg, "--add-empty") == 0 && i + 3 < argc) {
        const size_t track = (size_t)std::atoi(argv[i + 1]);
        const float mix_duration = std::strtof(argv[i + 2], nullptr);
        const float delay = std::strtof(argv[i + 3], nullptr);
        last_entry = spine_animation_state_add_empty_animation(state, track, mix_duration, delay);
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

      if (std::strcmp(arg, "--entry-mix-blend") == 0 && i + 1 < argc) {
        if (!last_entry) {
          std::cerr << "--entry-mix-blend requires a preceding --set/--add command\n";
          return 2;
        }
        spine_mix_blend blend;
        if (!parse_mix_blend(argv[i + 1], &blend)) {
          std::cerr << "--entry-mix-blend must be setup|first|replace|add\n";
          return 2;
        }
        spine_track_entry_set_mix_blend(last_entry, blend);
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
        debug_dump_bones("after-apply-before-world", skeleton, total_time + dt);
        spine_skeleton_update(skeleton, dt);
        spine_skeleton_update_world_transform(skeleton, physics);
        debug_dump_bones("after-world", skeleton, total_time + dt);
        total_time += dt;
        i += 1;
        continue;
      }

      std::cerr << "unknown/invalid command: " << arg << "\n";
      usage();
      return 2;
    }

    animation = "<scenario>";
    time = total_time;
  }

  // Bones.
  spine_array_bone bones = spine_skeleton_get_bones(skeleton);
  const size_t nb = spine_array_bone_size(bones);
  spine_bone *bones_buf = spine_array_bone_buffer(bones);

  std::cout << "{\"mode\":\"" << (legacy_mode ? "legacy" : "scenario") << "\",\"animation\":\""
            << json_escape(animation) << "\",\"time\":" << time << ",\"yDown\":" << y_down
            << ",\"bones\":[";
  for (size_t i = 0; i < nb; i++) {
    spine_bone bone = bones_buf[i];
    spine_bone_data bd = spine_bone_get_data(bone);
    const char *name = bd ? spine_bone_data_get_name(bd) : "<unknown>";
    spine_bone_pose pose = spine_bone_get_applied_pose(bone);

    std::cout << "{\"i\":" << i << ",\"name\":\"" << json_escape(name) << "\",\"active\":"
              << (spine_bone_is_active(bone) ? 1 : 0) << ",\"world\":{"
              << "\"a\":" << spine_bone_pose_get_a(pose) << ",\"b\":" << spine_bone_pose_get_b(pose)
              << ",\"c\":" << spine_bone_pose_get_c(pose) << ",\"d\":" << spine_bone_pose_get_d(pose)
              << ",\"x\":" << spine_bone_pose_get_world_x(pose) << ",\"y\":" << spine_bone_pose_get_world_y(pose)
              << "},\"applied\":{"
              << "\"x\":" << spine_bone_pose_get_x(pose) << ",\"y\":" << spine_bone_pose_get_y(pose)
              << ",\"rotation\":" << spine_bone_pose_get_rotation(pose)
              << ",\"scaleX\":" << spine_bone_pose_get_scale_x(pose) << ",\"scaleY\":" << spine_bone_pose_get_scale_y(pose)
              << ",\"shearX\":" << spine_bone_pose_get_shear_x(pose) << ",\"shearY\":" << spine_bone_pose_get_shear_y(pose)
              << "}}";
    if (i + 1 != nb) std::cout << ",";
  }

  // Slots.
  spine_array_slot slots = spine_skeleton_get_slots(skeleton);
  const size_t ns = spine_array_slot_size(slots);
  spine_slot *slots_buf = spine_array_slot_buffer(slots);

  std::cout << "],\"slots\":[";
  for (size_t i = 0; i < ns; i++) {
    spine_slot slot = slots_buf[i];
    spine_slot_data sd = spine_slot_get_data(slot);
    const char *slot_name = sd ? spine_slot_data_get_name(sd) : "<unknown>";
    spine_slot_pose sp = spine_slot_get_applied_pose(slot);
    spine_color c = spine_slot_pose_get_color(sp);
    spine_color dc = spine_slot_pose_get_dark_color(sp);
    const int has_dark = spine_slot_pose_has_dark_color(sp) ? 1 : 0;
    const int sequence_index = spine_slot_pose_get_sequence_index(sp);

    spine_attachment att = spine_slot_pose_get_attachment(sp);
    const char *att_name = att ? spine_attachment_get_name(att) : nullptr;
    const AttachmentTypeInfo ati = attachment_type_info(att);

    std::cout << "{\"i\":" << i << ",\"name\":\"" << json_escape(slot_name) << "\",\"color\":["
              << spine_color_get_r(c) << "," << spine_color_get_g(c) << "," << spine_color_get_b(c)
              << "," << spine_color_get_a(c) << "],\"hasDark\":" << has_dark << ",\"darkColor\":["
              << spine_color_get_r(dc) << "," << spine_color_get_g(dc) << "," << spine_color_get_b(dc)
              << "," << spine_color_get_a(dc) << "],\"sequenceIndex\":" << sequence_index
              << ",\"attachment\":";
    if (att) {
      std::cout << "{\"name\":\"" << json_escape(att_name ? att_name : "") << "\",\"type\":"
                << ati.type << ",\"typeName\":\"" << ati.name << "\"}";
    } else {
      std::cout << "null";
    }
    std::cout << "}";
    if (i + 1 != ns) std::cout << ",";
  }

  // Draw order as slot data indices.
  spine_array_slot draw_order = spine_skeleton_get_draw_order(skeleton);
  const size_t nd = spine_array_slot_size(draw_order);
  spine_slot *draw_buf = spine_array_slot_buffer(draw_order);
  std::cout << "],\"drawOrder\":[";
  for (size_t i = 0; i < nd; i++) {
    spine_slot ds = draw_buf[i];
    spine_slot_data dsd = spine_slot_get_data(ds);
    const int idx = dsd ? spine_slot_data_get_index(dsd) : -1;
    std::cout << idx;
    if (i + 1 != nd) std::cout << ",";
  }

  // Constraints (runtime values).
  spine_array_constraint constraints = spine_skeleton_get_constraints(skeleton);
  const size_t nc = spine_array_constraint_size(constraints);
  spine_constraint *constraints_buf = spine_array_constraint_buffer(constraints);

  // NOTE: Spine 4.3's C API exposes `isActive()` via `PosedActive`, but the actual runtime gating
  // flag used by `Skeleton::updateCache` lives in `Constraint::_active` (different field).
  // The simplest correct oracle is: a constraint is "active" iff it was inserted into the
  // skeleton update cache.
  spine_array_update update_cache = spine_skeleton_get_update_cache(skeleton);
  const size_t nuc = spine_array_update_size(update_cache);
  spine_update *update_cache_buf = spine_array_update_buffer(update_cache);
  std::unordered_set<const void *> update_cache_set;
  update_cache_set.reserve(nuc * 2 + 8);
  for (size_t i = 0; i < nuc; i++) {
    update_cache_set.insert((const void *)update_cache_buf[i]);
  }

  std::cout << "],\"ikConstraints\":[";
  bool first = true;
  int ik_i = 0;
  for (size_t i = 0; i < nc; i++) {
    spine_constraint cst = constraints_buf[i];
    const spine_rtti rt = spine_constraint_get_rtti(cst);
    if (!spine_rtti_instance_of(rt, spine_ik_constraint_rtti())) continue;
    spine_ik_constraint_base ik = spine_constraint_cast_to_ik_constraint_base(cst);
    spine_ik_constraint_data cd = spine_ik_constraint_base_get_data(ik);
    const char *name = cd ? spine_ik_constraint_data_get_name(cd) : "<unknown>";
    spine_ik_constraint_pose pose = spine_ik_constraint_base_get_applied_pose(ik);
    const spine_update u = spine_constraint_cast_to_update(cst);
    const int active = update_cache_set.count((const void *)u) ? 1 : 0;
    if (!first) std::cout << ",";
    first = false;
    std::cout << "{\"i\":" << ik_i++ << ",\"name\":\"" << json_escape(name) << "\""
              << ",\"mix\":" << spine_ik_constraint_pose_get_mix(pose)
              << ",\"softness\":" << spine_ik_constraint_pose_get_softness(pose)
              << ",\"bendDirection\":" << spine_ik_constraint_pose_get_bend_direction(pose)
              << ",\"active\":" << active
              << "}";
  }

  std::cout << "],\"transformConstraints\":[";
  first = true;
  int tx_i = 0;
  for (size_t i = 0; i < nc; i++) {
    spine_constraint cst = constraints_buf[i];
    const spine_rtti rt = spine_constraint_get_rtti(cst);
    if (!spine_rtti_instance_of(rt, spine_transform_constraint_rtti())) continue;
    spine_transform_constraint_base tc = spine_constraint_cast_to_transform_constraint_base(cst);
    spine_transform_constraint_data cd = spine_transform_constraint_base_get_data(tc);
    const char *name = cd ? spine_transform_constraint_data_get_name(cd) : "<unknown>";
    spine_transform_constraint_pose pose = spine_transform_constraint_base_get_applied_pose(tc);
    const spine_update u = spine_constraint_cast_to_update(cst);
    const int active = update_cache_set.count((const void *)u) ? 1 : 0;
    if (!first) std::cout << ",";
    first = false;
    std::cout << "{\"i\":" << tx_i++ << ",\"name\":\"" << json_escape(name) << "\""
              << ",\"mixRotate\":" << spine_transform_constraint_pose_get_mix_rotate(pose)
              << ",\"mixX\":" << spine_transform_constraint_pose_get_mix_x(pose)
              << ",\"mixY\":" << spine_transform_constraint_pose_get_mix_y(pose)
              << ",\"mixScaleX\":" << spine_transform_constraint_pose_get_mix_scale_x(pose)
              << ",\"mixScaleY\":" << spine_transform_constraint_pose_get_mix_scale_y(pose)
              << ",\"mixShearY\":" << spine_transform_constraint_pose_get_mix_shear_y(pose)
              << ",\"active\":" << active
              << "}";
  }

  std::cout << "],\"pathConstraints\":[";
  first = true;
  int pc_i = 0;
  for (size_t i = 0; i < nc; i++) {
    spine_constraint cst = constraints_buf[i];
    const spine_rtti rt = spine_constraint_get_rtti(cst);
    if (!spine_rtti_instance_of(rt, spine_path_constraint_rtti())) continue;
    spine_path_constraint_base pc = spine_constraint_cast_to_path_constraint_base(cst);
    spine_path_constraint_data cd = spine_path_constraint_base_get_data(pc);
    const char *name = cd ? spine_path_constraint_data_get_name(cd) : "<unknown>";
    spine_path_constraint_pose pose = spine_path_constraint_base_get_applied_pose(pc);
    const spine_update u = spine_constraint_cast_to_update(cst);
    const int active = update_cache_set.count((const void *)u) ? 1 : 0;
    if (!first) std::cout << ",";
    first = false;
    std::cout << "{\"i\":" << pc_i++ << ",\"name\":\"" << json_escape(name) << "\""
              << ",\"position\":" << spine_path_constraint_pose_get_position(pose)
              << ",\"spacing\":" << spine_path_constraint_pose_get_spacing(pose)
              << ",\"mixRotate\":" << spine_path_constraint_pose_get_mix_rotate(pose)
              << ",\"mixX\":" << spine_path_constraint_pose_get_mix_x(pose)
              << ",\"mixY\":" << spine_path_constraint_pose_get_mix_y(pose)
              << ",\"active\":" << active
              << "}";
  }

  // Physics constraints.
  spine_array_physics_constraint phys = spine_skeleton_get_physics_constraints(skeleton);
  const size_t nphys = spine_array_physics_constraint_size(phys);
  spine_physics_constraint *phys_buf = spine_array_physics_constraint_buffer(phys);
  std::cout << "],\"physicsConstraints\":[";
  for (size_t i = 0; i < nphys; i++) {
    spine_physics_constraint cst = phys_buf[i];
    spine_physics_constraint_data cd = spine_physics_constraint_get_data(cst);
    const char *name = cd ? spine_physics_constraint_data_get_name(cd) : "<unknown>";
    spine_physics_constraint_pose pose = spine_physics_constraint_get_applied_pose(cst);

    const auto *cpp = reinterpret_cast<const spine::PhysicsConstraint *>(cst);
    const int reset = cpp->_reset ? 1 : 0;
    const spine_update u = spine_physics_constraint_cast_to_update(cst);
    const int active = update_cache_set.count((const void *)u) ? 1 : 0;

    std::cout << "{\"i\":" << i << ",\"name\":\"" << json_escape(name) << "\""
              << ",\"inertia\":" << spine_physics_constraint_pose_get_inertia(pose)
              << ",\"strength\":" << spine_physics_constraint_pose_get_strength(pose)
              << ",\"damping\":" << spine_physics_constraint_pose_get_damping(pose)
              << ",\"massInverse\":" << spine_physics_constraint_pose_get_mass_inverse(pose)
              << ",\"wind\":" << spine_physics_constraint_pose_get_wind(pose)
              << ",\"gravity\":" << spine_physics_constraint_pose_get_gravity(pose)
              << ",\"mix\":" << spine_physics_constraint_pose_get_mix(pose)
              << ",\"reset\":" << reset
              << ",\"ux\":" << cpp->_ux
              << ",\"uy\":" << cpp->_uy
              << ",\"cx\":" << cpp->_cx
              << ",\"cy\":" << cpp->_cy
              << ",\"tx\":" << cpp->_tx
              << ",\"ty\":" << cpp->_ty
              << ",\"xOffset\":" << cpp->_xOffset
              << ",\"xVelocity\":" << cpp->_xVelocity
              << ",\"yOffset\":" << cpp->_yOffset
              << ",\"yVelocity\":" << cpp->_yVelocity
              << ",\"rotateOffset\":" << cpp->_rotateOffset
              << ",\"rotateVelocity\":" << cpp->_rotateVelocity
              << ",\"scaleOffset\":" << cpp->_scaleOffset
              << ",\"scaleVelocity\":" << cpp->_scaleVelocity
              << ",\"remaining\":" << cpp->_remaining
              << ",\"lastTime\":" << cpp->_lastTime
              << ",\"active\":" << active
              << "}";
    if (i + 1 != nphys) std::cout << ",";
  }

  std::cout << "]";
  if ((dump_slot_vertices && dump_slot_vertices[0]) || dump_update_cache) {
    std::cout << ",\"debug\":{";
    bool first_debug = true;

    if (dump_slot_vertices && dump_slot_vertices[0]) {
      spine_slot slot = spine_skeleton_find_slot(skeleton, dump_slot_vertices);
      std::cout << "\"slot\":\"" << json_escape(dump_slot_vertices) << "\""
                << ",\"worldVertices\":";
      first_debug = false;
      if (slot) {
        spine_slot_pose sp = spine_slot_get_applied_pose(slot);
        spine_attachment att = spine_slot_pose_get_attachment(sp);
        if (att) {
          const spine_rtti rt = spine_attachment_get_rtti(att);
          const bool is_vertex =
              spine_rtti_instance_of(rt, spine_mesh_attachment_rtti()) ||
              spine_rtti_instance_of(rt, spine_path_attachment_rtti()) ||
              spine_rtti_instance_of(rt, spine_bounding_box_attachment_rtti()) ||
              spine_rtti_instance_of(rt, spine_clipping_attachment_rtti());
          if (is_vertex) {
            spine_vertex_attachment va = spine_attachment_cast_to_vertex_attachment(att);
            const size_t len = spine_vertex_attachment_get_world_vertices_length(va);
            std::vector<float> verts(len > 0 ? len : 0);
            if (len > 0) {
              spine_vertex_attachment_compute_world_vertices_1(
                  va, skeleton, slot, 0, len, verts.data(), 0, 2);
            }
            std::cout << "[";
            for (size_t j = 0; j < len; j++) {
              std::cout << verts[j];
              if (j + 1 != len) std::cout << ",";
            }
            std::cout << "]";
          } else {
            std::cout << "null";
          }
        } else {
          std::cout << "null";
        }
      } else {
        std::cout << "null";
      }
    }

    if (dump_update_cache) {
      if (!first_debug) std::cout << ",";
      first_debug = false;

      std::unordered_map<const void *, std::string> update_names;
      update_names.reserve(nb + nc + 8);
      for (size_t i = 0; i < nb; i++) {
        spine_bone bone = bones_buf[i];
        spine_bone_data bd = spine_bone_get_data(bone);
        const char *name = bd ? spine_bone_data_get_name(bd) : "<unknown>";
        spine_bone_pose pose = spine_bone_get_applied_pose(bone);
        const spine_update u = spine_bone_pose_cast_to_update(pose);
        update_names[(const void *)u] = std::string("bone ") + name;
      }
      for (size_t i = 0; i < nc; i++) {
        spine_constraint cst = constraints_buf[i];
        spine_constraint_data cd = spine_constraint_get_data(cst);
        const char *name = cd ? spine_constraint_data_get_name(cd) : "<unknown>";
        const spine_rtti rt = spine_constraint_get_rtti(cst);
        std::string prefix = "constraint ";
        if (spine_rtti_instance_of(rt, spine_ik_constraint_rtti())) prefix = "ik ";
        else if (spine_rtti_instance_of(rt, spine_transform_constraint_rtti())) prefix = "transform ";
        else if (spine_rtti_instance_of(rt, spine_path_constraint_rtti())) prefix = "path ";
        else if (spine_rtti_instance_of(rt, spine_physics_constraint_rtti())) prefix = "physics ";
        else if (spine_rtti_instance_of(rt, spine_slider_rtti())) prefix = "slider ";
        const spine_update u = spine_constraint_cast_to_update(cst);
        update_names[(const void *)u] = prefix + name;
      }

      std::cout << "\"updateCache\":[";
      for (size_t i = 0; i < nuc; i++) {
        const void *u = (const void *)update_cache_buf[i];
        auto it = update_names.find(u);
        const std::string label = (it == update_names.end()) ? std::string("<unknown>") : it->second;
        std::cout << "\"" << json_escape(label.c_str()) << "\"";
        if (i + 1 != nuc) std::cout << ",";
      }
      std::cout << "]";
    }

    std::cout << "}";
  }
  std::cout << "}\n";

  spine_skeleton_drawable_dispose(drawable);
  spine_skeleton_data_result_dispose(data_result);
  spine_atlas_dispose(atlas);
  spine_atlas_result_dispose(atlas_result);
  return 0;
}
