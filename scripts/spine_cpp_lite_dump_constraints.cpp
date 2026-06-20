#include <cstdlib>
#include <cstring>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

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

static bool ends_with(const std::string &s, const char *suffix) {
  const size_t n = std::strlen(suffix);
  if (s.size() < n) return false;
  return s.compare(s.size() - n, n, suffix) == 0;
}

static void print_float_array(const char *label, spine_array_float array) {
  const size_t n = spine_array_float_size(array);
  const float *buf = spine_array_float_buffer(array);
  std::cout << "    " << label << "=[";
  for (size_t i = 0; i < n; i++) {
    if (i) std::cout << ",";
    std::cout << buf[i];
  }
  std::cout << "]\n";
}

static void usage() {
  std::cerr << "Usage:\n"
               "  spine_cpp_lite_dump_constraints <atlas.atlas> <skeleton.(json|skel)> [--y-down 0|1] [--dump-animation <name>]\n";
}

int main(int argc, char **argv) {
  if (argc < 3) {
    usage();
    return 2;
  }

  const char *atlas_path = argv[1];
  const char *skeleton_path = argv[2];

  int y_down = 0;
  const char *dump_animation = nullptr;
  for (int i = 3; i < argc; i++) {
    if (std::strcmp(argv[i], "--y-down") == 0 && i + 1 < argc) {
      y_down = std::atoi(argv[i + 1]) ? 1 : 0;
      i++;
      continue;
    }
    if (std::strcmp(argv[i], "--dump-animation") == 0 && i + 1 < argc) {
      dump_animation = argv[i + 1];
      i++;
      continue;
    }
  }

  spine_bone_set_y_down(y_down ? true : false);

  spine_atlas_result atlas_result = nullptr;
  spine_atlas atlas = nullptr;
  {
    std::string atlas_text = read_file(atlas_path);
    atlas_result = spine_atlas_load(atlas_text.c_str());
    if (!atlas_result) {
      std::cerr << "spine_atlas_load failed\n";
      return 2;
    }
    const char *err = spine_atlas_result_get_error(atlas_result);
    if (err && err[0]) {
      std::cerr << "atlas error: " << err << "\n";
      return 2;
    }
    atlas = spine_atlas_result_get_atlas(atlas_result);
    if (!atlas) {
      std::cerr << "missing atlas\n";
      return 2;
    }
  }

  spine_skeleton_data_result data_result = nullptr;
  spine_skeleton_data data = nullptr;
  {
    const std::string sk_path(skeleton_path);
    if (ends_with(sk_path, ".skel")) {
      std::string bytes = read_file(skeleton_path);
      data_result = spine_skeleton_data_load_binary(
          atlas, reinterpret_cast<const uint8_t *>(bytes.data()), (int32_t)bytes.size(), skeleton_path);
    } else {
      std::string json_text = read_file(skeleton_path);
      data_result = spine_skeleton_data_load_json(atlas, json_text.c_str(), skeleton_path);
    }

    if (!data_result) {
      std::cerr << "spine_skeleton_data_load_* failed\n";
      return 2;
    }
    const char *err = spine_skeleton_data_result_get_error(data_result);
    if (err && err[0]) {
      std::cerr << "skeleton data error: " << err << "\n";
      return 2;
    }
    data = spine_skeleton_data_result_get_data(data_result);
    if (!data) {
      std::cerr << "missing skeleton data\n";
      return 2;
    }
  }

  spine_array_constraint_data constraints = spine_skeleton_data_get_constraints(data);
  const size_t n = spine_array_constraint_data_size(constraints);
  spine_constraint_data *buf = spine_array_constraint_data_buffer(constraints);

  size_t num_ik = 0;
  size_t num_transform = 0;
  size_t num_path = 0;
  size_t num_physics = 0;
  size_t num_slider = 0;
  for (size_t i = 0; i < n; i++) {
    spine_constraint_data c = buf[i];
    const spine_rtti rt = spine_constraint_data_get_rtti(c);
    if (spine_rtti_instance_of(rt, spine_ik_constraint_data_rtti())) num_ik++;
    else if (spine_rtti_instance_of(rt, spine_transform_constraint_data_rtti())) num_transform++;
    else if (spine_rtti_instance_of(rt, spine_path_constraint_data_rtti())) num_path++;
    else if (spine_rtti_instance_of(rt, spine_physics_constraint_data_rtti())) num_physics++;
    else if (spine_rtti_instance_of(rt, spine_slider_data_rtti())) num_slider++;
  }

  std::cout << "Constraints total: " << n << "\n";
  std::cout << "IK constraints: " << num_ik << "\n";
  std::cout << "Transform constraints: " << num_transform << "\n";
  std::cout << "Path constraints: " << num_path << "\n";
  std::cout << "Physics constraints: " << num_physics << "\n";
  std::cout << "Slider constraints: " << num_slider << "\n";

  for (size_t i = 0; i < n; i++) {
    spine_constraint_data c = buf[i];
    const spine_rtti rt = spine_constraint_data_get_rtti(c);
    const char *name = spine_constraint_data_get_name(c);

    if (spine_rtti_instance_of(rt, spine_ik_constraint_data_rtti())) {
      spine_ik_constraint_data ik = spine_constraint_data_cast_to_ik_constraint_data(c);
      spine_ik_constraint_pose setup = spine_ik_constraint_data_get_setup_pose(ik);
      std::cout << "  [ik] " << (name ? name : "?")
                << " mix=" << spine_ik_constraint_pose_get_mix(setup)
                << " softness=" << spine_ik_constraint_pose_get_softness(setup)
                << " bend=" << spine_ik_constraint_pose_get_bend_direction(setup)
                << " compress=" << (spine_ik_constraint_pose_get_compress(setup) ? 1 : 0)
                << " stretch=" << (spine_ik_constraint_pose_get_stretch(setup) ? 1 : 0)
                << " scaleYMode=" << (int)spine_ik_constraint_data_get_scale_y_mode(ik)
                << " skin=" << (spine_constraint_data_get_skin_required(c) ? 1 : 0)
                << "\n";
    } else if (spine_rtti_instance_of(rt, spine_transform_constraint_data_rtti())) {
      spine_transform_constraint_data tr = spine_constraint_data_cast_to_transform_constraint_data(c);
      spine_transform_constraint_pose setup = spine_transform_constraint_data_get_setup_pose(tr);
      std::cout << "  [transform] " << (name ? name : "?")
                << " mixRotate=" << spine_transform_constraint_pose_get_mix_rotate(setup)
                << " mixX=" << spine_transform_constraint_pose_get_mix_x(setup)
                << " mixY=" << spine_transform_constraint_pose_get_mix_y(setup)
                << " mixScaleX=" << spine_transform_constraint_pose_get_mix_scale_x(setup)
                << " mixScaleY=" << spine_transform_constraint_pose_get_mix_scale_y(setup)
                << " mixShearY=" << spine_transform_constraint_pose_get_mix_shear_y(setup)
                << " localSource=" << (spine_transform_constraint_data_get_local_source(tr) ? 1 : 0)
                << " localTarget=" << (spine_transform_constraint_data_get_local_target(tr) ? 1 : 0)
                << " additive=" << (spine_transform_constraint_data_get_additive(tr) ? 1 : 0)
                << " clamp=" << (spine_transform_constraint_data_get_clamp(tr) ? 1 : 0)
                << " skin=" << (spine_constraint_data_get_skin_required(c) ? 1 : 0)
                << "\n";
    } else if (spine_rtti_instance_of(rt, spine_path_constraint_data_rtti())) {
      spine_path_constraint_data pc = spine_constraint_data_cast_to_path_constraint_data(c);
      const spine_position_mode pm = spine_path_constraint_data_get_position_mode(pc);
      const spine_spacing_mode sm = spine_path_constraint_data_get_spacing_mode(pc);
      const spine_rotate_mode rm = spine_path_constraint_data_get_rotate_mode(pc);
      spine_path_constraint_pose setup = spine_path_constraint_data_get_setup_pose(pc);
      std::cout << "  [path] " << (name ? name : "?")
                << " position=" << spine_path_constraint_pose_get_position(setup)
                << " spacing=" << spine_path_constraint_pose_get_spacing(setup)
                << " mixRotate=" << spine_path_constraint_pose_get_mix_rotate(setup)
                << " mixX=" << spine_path_constraint_pose_get_mix_x(setup)
                << " mixY=" << spine_path_constraint_pose_get_mix_y(setup)
                << " positionMode=" << (int)pm
                << " spacingMode=" << (int)sm
                << " rotateMode=" << (int)rm
                << " skin=" << (spine_constraint_data_get_skin_required(c) ? 1 : 0)
                << "\n";
    } else if (spine_rtti_instance_of(rt, spine_slider_data_rtti())) {
      spine_slider_data sd = spine_constraint_data_cast_to_slider_data(c);
      spine_slider_pose setup = spine_slider_data_get_setup_pose(sd);
      spine_animation anim = spine_slider_data_get_animation(sd);
      const char *anim_name = anim ? spine_animation_get_name(anim) : nullptr;
      spine_bone_data bone = spine_slider_data_get_bone(sd);
      const char *bone_name = bone ? spine_bone_data_get_name(bone) : nullptr;
      const bool has_property = spine_slider_data_get_property(sd) != nullptr;
      std::cout << "  [slider] " << (name ? name : "?")
                << " animation=" << (anim_name ? anim_name : "<null>")
                << " time=" << spine_slider_pose_get_time(setup)
                << " mix=" << spine_slider_pose_get_mix(setup)
                << " loop=" << (spine_slider_data_get_loop(sd) ? 1 : 0)
                << " additive=" << (spine_slider_data_get_additive(sd) ? 1 : 0)
                << " bone=" << (bone_name ? bone_name : "<none>")
                << " property=" << (has_property ? "1" : "0")
                << " scale=" << spine_slider_data_get_scale(sd)
                << " offset=" << spine_slider_data_get_offset(sd)
                << " local=" << (spine_slider_data_get_local(sd) ? 1 : 0)
                << " skin=" << (spine_constraint_data_get_skin_required(c) ? 1 : 0)
                << "\n";
    }
  }

  spine_skeleton_data_result_dispose(data_result);
  spine_atlas_dispose(atlas);
  spine_atlas_result_dispose(atlas_result);

  if (dump_animation != nullptr) {
    spine_animation anim = spine_skeleton_data_find_animation(data, dump_animation);
    if (!anim) {
      std::cerr << "Missing animation: " << dump_animation << "\n";
      return 2;
    }

    spine_array_slot_data slots = spine_skeleton_data_get_slots(data);
    const int slot_count = (int) spine_array_slot_data_size(slots);
    spine_array_constraint_data all_constraints = spine_skeleton_data_get_constraints(data);
    const int constraint_count = (int) spine_array_constraint_data_size(all_constraints);

    spine_array_timeline timelines = spine_animation_get_timelines(anim);
    const size_t tn = spine_array_timeline_size(timelines);
    spine_timeline *tbuf = spine_array_timeline_buffer(timelines);
    std::cout << "Animation: " << spine_animation_get_name(anim) << "\n";
    std::cout << "Timelines: " << tn << "\n";
    for (size_t i = 0; i < tn; i++) {
      spine_timeline t = tbuf[i];
      spine_rtti rt = spine_timeline_get_rtti(t);
      const char *class_name = spine_rtti_get_class_name(rt);
      std::cout << "  [" << i << "] " << (class_name ? class_name : "<unknown>");

      if (spine_rtti_instance_of(rt, spine_slot_timeline_rtti())) {
        spine_slot_timeline st = nullptr;
        if (spine_rtti_instance_of(rt, spine_slot_curve_timeline_rtti())) {
          spine_slot_curve_timeline sct = spine_timeline_cast_to_slot_curve_timeline(t);
          st = spine_slot_curve_timeline_cast_to_slot_timeline(sct);
        } else if (spine_rtti_instance_of(rt, spine_attachment_timeline_rtti())) {
          spine_attachment_timeline at = spine_timeline_cast_to_attachment_timeline(t);
          st = spine_attachment_timeline_cast_to_slot_timeline(at);
        } else if (spine_rtti_instance_of(rt, spine_deform_timeline_rtti())) {
          spine_deform_timeline dt = spine_timeline_cast_to_deform_timeline(t);
          st = spine_deform_timeline_cast_to_slot_timeline(dt);
        } else if (spine_rtti_instance_of(rt, spine_sequence_timeline_rtti())) {
          spine_sequence_timeline qt = spine_timeline_cast_to_sequence_timeline(t);
          st = spine_sequence_timeline_cast_to_slot_timeline(qt);
        } else if (spine_rtti_instance_of(rt, spine_alpha_timeline_rtti())) {
          spine_alpha_timeline at = spine_timeline_cast_to_alpha_timeline(t);
          st = spine_alpha_timeline_cast_to_slot_timeline(at);
        }

        if (st) {
          const int idx = spine_slot_timeline_get_slot_index(st);
          std::cout << " slotIndex=" << idx;
          if (idx < 0 || idx >= slot_count) std::cout << " (OOB!)";
        } else {
          std::cout << " slotIndex=<unavailable>";
        }
      } else if (spine_rtti_instance_of(rt, spine_constraint_timeline_rtti())) {
        spine_constraint_timeline1 ct1 = spine_timeline_cast_to_constraint_timeline1(t);
        spine_constraint_timeline ct = spine_constraint_timeline1_cast_to_constraint_timeline(ct1);
        const int idx = spine_constraint_timeline_get_constraint_index(ct);
        std::cout << " constraintIndex=" << idx;
        if (idx < -1 || idx >= constraint_count) std::cout << " (OOB!)";
      } else if (spine_rtti_instance_of(rt, spine_bone_timeline1_rtti())) {
        spine_bone_timeline1 bt = spine_timeline_cast_to_bone_timeline1(t);
        const int idx = spine_bone_timeline1_get_bone_index(bt);
        spine_array_bone_data bones = spine_skeleton_data_get_bones(data);
        const int bone_count = (int) spine_array_bone_data_size(bones);
        std::cout << " boneIndex=" << idx;
        if (idx < 0 || idx >= bone_count) std::cout << " (OOB!)";
      } else if (spine_rtti_instance_of(rt, spine_bone_timeline2_rtti())) {
        spine_bone_timeline2 bt = spine_timeline_cast_to_bone_timeline2(t);
        const int idx = spine_bone_timeline2_get_bone_index(bt);
        spine_array_bone_data bones = spine_skeleton_data_get_bones(data);
        const int bone_count = (int) spine_array_bone_data_size(bones);
        std::cout << " boneIndex=" << idx;
        if (idx < 0 || idx >= bone_count) std::cout << " (OOB!)";
      }

      std::cout << "\n";

      const char *timeline_name = nullptr;
      if (spine_rtti_instance_of(rt, spine_bone_timeline1_rtti())) {
        spine_bone_timeline1 bt = spine_timeline_cast_to_bone_timeline1(t);
        const int bone_idx = spine_bone_timeline1_get_bone_index(bt);
        spine_array_bone_data bones = spine_skeleton_data_get_bones(data);
        if (bone_idx >= 0 && (size_t)bone_idx < spine_array_bone_data_size(bones)) {
          timeline_name = spine_bone_data_get_name(spine_array_bone_data_buffer(bones)[bone_idx]);
        }
        if (timeline_name) {
          std::cout << "    kind=bone1 name=" << timeline_name << "\n";
          print_float_array("frames", spine_bone_timeline1_get_frames(bt));
          print_float_array("curves", spine_bone_timeline1_get_curves(bt));
        }
      } else if (spine_rtti_instance_of(rt, spine_bone_timeline2_rtti())) {
        spine_bone_timeline2 bt = spine_timeline_cast_to_bone_timeline2(t);
        const int bone_idx = spine_bone_timeline2_get_bone_index(bt);
        spine_array_bone_data bones = spine_skeleton_data_get_bones(data);
        if (bone_idx >= 0 && (size_t)bone_idx < spine_array_bone_data_size(bones)) {
          timeline_name = spine_bone_data_get_name(spine_array_bone_data_buffer(bones)[bone_idx]);
        }
        if (timeline_name) {
          std::cout << "    kind=bone2 name=" << timeline_name << "\n";
          print_float_array("frames", spine_bone_timeline2_get_frames(bt));
          print_float_array("curves", spine_bone_timeline2_get_curves(bt));
        }
      } else if (spine_rtti_instance_of(rt, spine_constraint_timeline_rtti())) {
        spine_constraint_timeline1 ct1 = spine_timeline_cast_to_constraint_timeline1(t);
        spine_constraint_timeline ct = spine_constraint_timeline1_cast_to_constraint_timeline(ct1);
        const int idx = spine_constraint_timeline_get_constraint_index(ct);
        if (idx >= 0 && idx < constraint_count) {
          timeline_name = spine_constraint_data_get_name(buf[idx]);
        }
        if (timeline_name) {
          std::cout << "    kind=constraint name=" << timeline_name << "\n";
          if (spine_rtti_instance_of(rt, spine_ik_constraint_timeline_rtti())) {
            spine_ik_constraint_timeline ikt = spine_timeline_cast_to_ik_constraint_timeline(t);
            print_float_array("frames", spine_ik_constraint_timeline_get_frames(ikt));
            print_float_array("curves", spine_ik_constraint_timeline_get_curves(ikt));
          } else if (spine_rtti_instance_of(rt, spine_path_constraint_position_timeline_rtti())) {
            spine_path_constraint_position_timeline pct = spine_timeline_cast_to_path_constraint_position_timeline(t);
            print_float_array("frames", spine_path_constraint_position_timeline_get_frames(pct));
            print_float_array("curves", spine_path_constraint_position_timeline_get_curves(pct));
          } else if (spine_rtti_instance_of(rt, spine_path_constraint_spacing_timeline_rtti())) {
            spine_path_constraint_spacing_timeline pst = spine_timeline_cast_to_path_constraint_spacing_timeline(t);
            print_float_array("frames", spine_path_constraint_spacing_timeline_get_frames(pst));
            print_float_array("curves", spine_path_constraint_spacing_timeline_get_curves(pst));
          } else if (spine_rtti_instance_of(rt, spine_path_constraint_mix_timeline_rtti())) {
            spine_path_constraint_mix_timeline pmt = spine_timeline_cast_to_path_constraint_mix_timeline(t);
            print_float_array("frames", spine_path_constraint_mix_timeline_get_frames(pmt));
            print_float_array("curves", spine_path_constraint_mix_timeline_get_curves(pmt));
          }
        }
      }
    }
  }

  return 0;
}
