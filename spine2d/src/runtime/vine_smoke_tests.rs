use crate::{Skeleton, SkeletonData};
use std::path::PathBuf;

fn vine_pro_json_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../assets/spine-runtimes/examples/vine/export/vine-pro.json"),
        manifest_dir.join("../third_party/spine-runtimes/examples/vine/export/vine-pro.json"),
        manifest_dir.join("../.cache/spine-runtimes/examples/vine/export/vine-pro.json"),
    ];
    for p in candidates {
        if p.is_file() {
            return p;
        }
    }
    panic!(
        "Upstream Spine vine example not found. Run `./scripts/import_spine_runtimes_examples.zsh --mode json` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

#[test]
fn vine_pro_json_parses_and_updates_world_transform() {
    let path = vine_pro_json_path();
    let json = std::fs::read_to_string(&path).expect("read vine-pro.json");
    let data = SkeletonData::from_json_str(&json).expect("parse vine-pro.json");

    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    for bone in &skeleton.bones {
        assert!(bone.world_x.is_finite());
        assert!(bone.world_y.is_finite());
        assert!(bone.a.is_finite());
        assert!(bone.b.is_finite());
        assert!(bone.c.is_finite());
        assert!(bone.d.is_finite());
    }
}
