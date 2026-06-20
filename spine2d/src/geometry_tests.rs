use crate::geometry::{SkeletonClipper, Triangulator};

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 0.001,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn triangulator_matches_spine_c_unit_test_rectangle() {
    let triangulator = Triangulator;

    let polygon = vec![0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 0.0, 100.0];
    let triangles = triangulator.triangulate(&polygon);
    assert_eq!(triangles, vec![3, 0, 1, 3, 1, 2]);

    let polys = triangulator.decompose(&polygon, &triangles);
    assert_eq!(polys.len(), 1);
    let poly0 = &polys[0];
    assert_eq!(poly0.len(), 8);
    assert_approx(poly0[0], 0.0);
    assert_approx(poly0[1], 100.0);
    assert_approx(poly0[2], 0.0);
    assert_approx(poly0[3], 0.0);
    assert_approx(poly0[4], 100.0);
    assert_approx(poly0[5], 0.0);
    assert_approx(poly0[6], 100.0);
    assert_approx(poly0[7], 100.0);
}

#[test]
fn skeleton_clipper_clip_triangles_matches_spine_c_unit_test() {
    let mut clipper = SkeletonClipper::default();

    let clip_polygon = vec![0.0, 50.0, 100.0, 50.0, 100.0, 70.0, 0.0, 70.0];
    assert!(clipper.clip_start(&clip_polygon, false, false));
    assert!(clipper.is_clipping());

    let vertices = vec![0.0, 0.0, 100.0, 0.0, 50.0, 150.0];
    let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
    let indices: Vec<u16> = vec![0, 1, 2];

    let (clipped_vertices, clipped_uvs, clipped_indices) =
        clipper.clip_triangles(&vertices, &indices, &uvs, 2);

    let expected_vertices = vec![
        76.666_664, 70.0, 23.333_334, 70.0, 16.666_672, 50.0, 83.333_33, 50.0,
    ];
    assert_eq!(clipped_vertices.len(), expected_vertices.len());
    for (actual, expected) in clipped_vertices.iter().copied().zip(expected_vertices) {
        assert_approx(actual, expected);
    }

    let expected_uvs = vec![
        0.766667, 0.466667, 0.233333, 0.466667, 0.166667, 0.333333, 0.833333, 0.333333,
    ];
    assert_eq!(clipped_uvs.len(), expected_uvs.len());
    for (actual, expected) in clipped_uvs.iter().copied().zip(expected_uvs) {
        assert_approx(actual, expected);
    }

    assert_eq!(clipped_indices, vec![0, 1, 2, 0, 2, 3]);
}

#[test]
fn skeleton_clipper_inverse_keeps_triangles_outside_clip_polygon() {
    let mut clipper = SkeletonClipper::default();

    let clip_polygon = vec![-1000.0, 50.0, 1000.0, 50.0, 1000.0, 70.0, -1000.0, 70.0];
    assert!(clipper.clip_start(&clip_polygon, false, true));

    let vertices = vec![0.0, 0.0, 100.0, 0.0, 50.0, 150.0];
    let uvs = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];
    let indices: Vec<u16> = vec![0, 1, 2];

    let (clipped_vertices, clipped_uvs, clipped_indices) =
        clipper.clip_triangles(&vertices, &indices, &uvs, 2);

    let expected_vertices = vec![
        76.666_664, 70.0, 50.0, 150.0, 23.333_334, 70.0, 16.666_672, 50.0, 0.0, 0.0, 100.0, 0.0,
        83.333_33, 50.0,
    ];
    assert_eq!(
        clipped_vertices.len(),
        expected_vertices.len(),
        "vertices: {clipped_vertices:?}"
    );
    for (actual, expected) in clipped_vertices.iter().copied().zip(expected_vertices) {
        assert_approx(actual, expected);
    }

    let expected_uvs = vec![
        0.766667, 0.466667, 0.5, 1.0, 0.233333, 0.466667, 0.166667, 0.333333, 0.0, 0.0, 1.0, 0.0,
        0.833333, 0.333333,
    ];
    assert_eq!(
        clipped_uvs.len(),
        expected_uvs.len(),
        "uvs: {clipped_uvs:?}"
    );
    for (actual, expected) in clipped_uvs.iter().copied().zip(expected_uvs) {
        assert_approx(actual, expected);
    }

    assert_eq!(clipped_indices, vec![0, 1, 2, 3, 4, 5, 3, 5, 6]);
}
