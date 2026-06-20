#![allow(dead_code)]

#[derive(Default)]
pub(crate) struct Triangulator;

impl Triangulator {
    pub(crate) fn triangulate(&self, vertices: &[f32]) -> Vec<u16> {
        let vertex_count = vertices.len() / 2;
        if vertex_count < 3 {
            return Vec::new();
        }

        let mut indices: Vec<usize> = (0..vertex_count).collect();
        let mut is_concave_flags: Vec<bool> = (0..vertex_count)
            .map(|i| is_concave_at(i, vertex_count, vertices, &indices))
            .collect();

        let mut triangles: Vec<u16> = Vec::with_capacity((vertex_count.saturating_sub(2)) * 3);
        let mut vertex_count = vertex_count;

        while vertex_count > 3 {
            let mut previous = vertex_count - 1;
            let mut next = 1usize;
            let mut i = 0usize;

            loop {
                if !is_concave_flags[i] {
                    let p1 = indices[previous] * 2;
                    let p2 = indices[i] * 2;
                    let p3 = indices[next] * 2;

                    let p1x = vertices[p1];
                    let p1y = vertices[p1 + 1];
                    let p2x = vertices[p2];
                    let p2y = vertices[p2 + 1];
                    let p3x = vertices[p3];
                    let p3y = vertices[p3 + 1];

                    let mut ear = true;
                    let mut ii = (next + 1) % vertex_count;
                    while ii != previous {
                        if is_concave_flags[ii] {
                            let v = indices[ii] * 2;
                            let vx = vertices[v];
                            let vy = vertices[v + 1];
                            if positive_area(p3x, p3y, p1x, p1y, vx, vy)
                                && positive_area(p1x, p1y, p2x, p2y, vx, vy)
                                && positive_area(p2x, p2y, p3x, p3y, vx, vy)
                            {
                                ear = false;
                                break;
                            }
                        }
                        ii = (ii + 1) % vertex_count;
                    }

                    if ear {
                        break;
                    }
                }

                if next == 0 {
                    while i > 0 {
                        if !is_concave_flags[i] {
                            break;
                        }
                        i -= 1;
                    }
                    break;
                }

                previous = i;
                i = next;
                next = (next + 1) % vertex_count;
            }

            triangles.push(indices[(vertex_count + i - 1) % vertex_count] as u16);
            triangles.push(indices[i] as u16);
            triangles.push(indices[(i + 1) % vertex_count] as u16);

            indices.remove(i);
            is_concave_flags.remove(i);
            vertex_count -= 1;

            let previous_index = (vertex_count + i - 1) % vertex_count;
            let next_index = if i == vertex_count { 0 } else { i };
            is_concave_flags[previous_index] =
                is_concave_at(previous_index, vertex_count, vertices, &indices);
            is_concave_flags[next_index] =
                is_concave_at(next_index, vertex_count, vertices, &indices);
        }

        if vertex_count == 3 {
            triangles.push(indices[2] as u16);
            triangles.push(indices[0] as u16);
            triangles.push(indices[1] as u16);
        }

        triangles
    }

    pub(crate) fn decompose(&self, vertices: &[f32], triangles: &[u16]) -> Vec<Vec<f32>> {
        let mut convex_polygons: Vec<Vec<f32>> = Vec::new();
        let mut convex_polygons_indices: Vec<Vec<usize>> = Vec::new();

        let mut polygon_indices: Vec<usize> = Vec::new();
        let mut polygon: Vec<f32> = Vec::new();

        let mut fan_base_index: Option<usize> = None;
        let mut last_winding: i32 = 0;

        for tri in triangles.chunks_exact(3) {
            let t1 = tri[0] as usize * 2;
            let t2 = tri[1] as usize * 2;
            let t3 = tri[2] as usize * 2;

            let x1 = vertices[t1];
            let y1 = vertices[t1 + 1];
            let x2 = vertices[t2];
            let y2 = vertices[t2 + 1];
            let x3 = vertices[t3];
            let y3 = vertices[t3 + 1];

            let mut merged = false;
            if fan_base_index == Some(t1) && polygon.len() >= 4 {
                let o = polygon.len() - 4;
                let winding1 = winding(
                    polygon[o],
                    polygon[o + 1],
                    polygon[o + 2],
                    polygon[o + 3],
                    x3,
                    y3,
                );
                let winding2 = winding(x3, y3, polygon[0], polygon[1], polygon[2], polygon[3]);
                if winding1 == last_winding && winding2 == last_winding {
                    polygon.push(x3);
                    polygon.push(y3);
                    polygon_indices.push(t3);
                    merged = true;
                }
            }

            if !merged {
                if !polygon.is_empty() {
                    convex_polygons.push(polygon);
                    convex_polygons_indices.push(polygon_indices);
                }

                polygon = vec![x1, y1, x2, y2, x3, y3];
                polygon_indices = vec![t1, t2, t3];
                last_winding = winding(x1, y1, x2, y2, x3, y3);
                fan_base_index = Some(t1);
            }
        }

        if !polygon.is_empty() {
            convex_polygons.push(polygon);
            convex_polygons_indices.push(polygon_indices);
        }

        let n = convex_polygons.len();
        for i in 0..n {
            if convex_polygons_indices[i].is_empty() {
                continue;
            }

            let first_index = convex_polygons_indices[i][0];
            let last_index = *convex_polygons_indices[i].last().unwrap();

            let poly_len = convex_polygons[i].len();
            let o = poly_len - 4;
            let mut prev_prev_x = convex_polygons[i][o];
            let mut prev_prev_y = convex_polygons[i][o + 1];
            let mut prev_x = convex_polygons[i][o + 2];
            let mut prev_y = convex_polygons[i][o + 3];
            let first_x = convex_polygons[i][0];
            let first_y = convex_polygons[i][1];
            let second_x = convex_polygons[i][2];
            let second_y = convex_polygons[i][3];
            let winding0 = winding(prev_prev_x, prev_prev_y, prev_x, prev_y, first_x, first_y);

            let mut ii = 0usize;
            while ii < n {
                if ii == i || convex_polygons_indices[ii].len() != 3 {
                    ii += 1;
                    continue;
                }

                let other_first_index = convex_polygons_indices[ii][0];
                let other_second_index = convex_polygons_indices[ii][1];
                let other_last_index = convex_polygons_indices[ii][2];
                if other_first_index != first_index || other_second_index != last_index {
                    ii += 1;
                    continue;
                }

                let other_len = convex_polygons[ii].len();
                let x3 = convex_polygons[ii][other_len - 2];
                let y3 = convex_polygons[ii][other_len - 1];

                let winding1 = winding(prev_prev_x, prev_prev_y, prev_x, prev_y, x3, y3);
                let winding2 = winding(x3, y3, first_x, first_y, second_x, second_y);
                if winding1 == winding0 && winding2 == winding0 {
                    convex_polygons[ii].clear();
                    convex_polygons_indices[ii].clear();
                    convex_polygons[i].push(x3);
                    convex_polygons[i].push(y3);
                    convex_polygons_indices[i].push(other_last_index);

                    prev_prev_x = prev_x;
                    prev_prev_y = prev_y;
                    prev_x = x3;
                    prev_y = y3;
                    ii = 0;
                    continue;
                }

                ii += 1;
            }
        }

        let mut out = Vec::new();
        for poly in convex_polygons {
            if !poly.is_empty() {
                out.push(poly);
            }
        }
        out
    }
}

fn positive_area(p1x: f32, p1y: f32, p2x: f32, p2y: f32, p3x: f32, p3y: f32) -> bool {
    p1x * (p3y - p2y) + p2x * (p1y - p3y) + p3x * (p2y - p1y) >= 0.0
}

fn is_concave_at(index: usize, vertex_count: usize, vertices: &[f32], indices: &[usize]) -> bool {
    let previous = indices[(vertex_count + index - 1) % vertex_count] * 2;
    let current = indices[index] * 2;
    let next = indices[(index + 1) % vertex_count] * 2;
    !positive_area(
        vertices[previous],
        vertices[previous + 1],
        vertices[current],
        vertices[current + 1],
        vertices[next],
        vertices[next + 1],
    )
}

fn winding(p1x: f32, p1y: f32, p2x: f32, p2y: f32, p3x: f32, p3y: f32) -> i32 {
    let px = p2x - p1x;
    let py = p2y - p1y;
    if p3x * py - p3y * px + px * p1y - p1x * py >= 0.0 {
        1
    } else {
        -1
    }
}

pub(crate) struct SkeletonClipper {
    triangulator: Triangulator,
    clipping_polygons: Vec<Vec<f32>>,
    inverse: bool,
    inverse_vertices: Vec<f32>,
    clip_output: Vec<f32>,
    scratch: Vec<f32>,
    scratch2: Vec<f32>,
}

impl Default for SkeletonClipper {
    fn default() -> Self {
        Self {
            triangulator: Triangulator,
            clipping_polygons: Vec::new(),
            inverse: false,
            inverse_vertices: Vec::new(),
            clip_output: Vec::new(),
            scratch: Vec::new(),
            scratch2: Vec::new(),
        }
    }
}

impl SkeletonClipper {
    pub(crate) fn clip_start(
        &mut self,
        polygon_vertices: &[f32],
        convex_flag: bool,
        inverse: bool,
    ) -> bool {
        if !self.clipping_polygons.is_empty() {
            return false;
        }
        if polygon_vertices.len() < 6 || polygon_vertices.len() % 2 != 0 {
            return false;
        }

        let mut clipping_polygon: Vec<f32> = polygon_vertices.to_vec();
        let convex = make_clockwise(&mut clipping_polygon);
        self.inverse = inverse;

        if convex || convex_flag || inverse {
            if !convex {
                make_convex(&mut clipping_polygon);
            }
            clipping_polygon.push(clipping_polygon[0]);
            clipping_polygon.push(clipping_polygon[1]);
            self.clipping_polygons = vec![clipping_polygon];
            return true;
        }

        let triangles = self.triangulator.triangulate(&clipping_polygon);
        let mut polygons = self.triangulator.decompose(&clipping_polygon, &triangles);
        for poly in &mut polygons {
            make_clockwise(poly);
            if poly.len() >= 2 {
                poly.push(poly[0]);
                poly.push(poly[1]);
            }
        }

        self.clipping_polygons = polygons;
        !self.clipping_polygons.is_empty()
    }

    pub(crate) fn clip_end(&mut self) {
        self.clipping_polygons.clear();
        self.inverse = false;
        self.inverse_vertices.clear();
        self.clip_output.clear();
        self.scratch.clear();
        self.scratch2.clear();
    }

    pub(crate) fn is_clipping(&self) -> bool {
        !self.clipping_polygons.is_empty()
    }

    pub(crate) fn clip_triangles(
        &mut self,
        vertices: &[f32],
        triangles: &[u16],
        uvs: &[f32],
        stride: usize,
    ) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
        let polygons = &self.clipping_polygons;
        if polygons.is_empty() {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        let mut clipped_vertices: Vec<f32> = Vec::new();
        let mut clipped_uvs: Vec<f32> = Vec::new();
        let mut clipped_triangles: Vec<u16> = Vec::new();

        let mut index: u16 = 0;

        if self.inverse {
            let polygon = polygons[0].clone();
            for tri in triangles.chunks_exact(3) {
                let (x1, y1, u1, v1) = read_vertex(vertices, uvs, tri[0], stride);
                let (x2, y2, u2, v2) = read_vertex(vertices, uvs, tri[1], stride);
                let (x3, y3, u3, v3) = read_vertex(vertices, uvs, tri[2], stride);

                self.clip_inverse(x1, y1, x2, y2, x3, y3, &polygon);
                if self.inverse_vertices.is_empty() {
                    continue;
                }

                let d0 = y2 - y3;
                let d1 = x3 - x2;
                let d2 = x1 - x3;
                let d4 = y3 - y1;
                let d = 1.0 / (d0 * d2 + d1 * (y1 - y3));

                let mut offset = 0usize;
                while offset < self.inverse_vertices.len() {
                    let polygon_size = self.inverse_vertices[offset] as usize;
                    offset += 1;
                    let vertex_count = polygon_size / 2;

                    let s = clipped_vertices.len();
                    clipped_vertices.resize(s + polygon_size, 0.0);
                    clipped_uvs.resize(s + polygon_size, 0.0);

                    let fragment = &self.inverse_vertices[offset..offset + polygon_size];
                    for (ii, xy) in fragment.chunks_exact(2).enumerate() {
                        let x = xy[0];
                        let y = xy[1];
                        let write = s + ii * 2;
                        clipped_vertices[write] = x;
                        clipped_vertices[write + 1] = y;

                        let c0 = x - x3;
                        let c1 = y - y3;
                        let a = (d0 * c0 + d1 * c1) * d;
                        let b = (d4 * c0 + d2 * c1) * d;
                        let c = 1.0 - a - b;
                        clipped_uvs[write] = u1 * a + u2 * b + u3 * c;
                        clipped_uvs[write + 1] = v1 * a + v2 * b + v3 * c;
                    }

                    clipped_triangles.reserve(3 * vertex_count.saturating_sub(2));
                    for ii in 1..vertex_count.saturating_sub(1) {
                        clipped_triangles.push(index);
                        clipped_triangles.push(index + ii as u16);
                        clipped_triangles.push(index + ii as u16 + 1);
                    }
                    index = index.wrapping_add(vertex_count as u16);
                    offset += polygon_size;
                }
            }

            return (clipped_vertices, clipped_uvs, clipped_triangles);
        }

        'outer: for tri in triangles.chunks_exact(3) {
            let (x1, y1, u1, v1) = read_vertex(vertices, uvs, tri[0], stride);
            let (x2, y2, u2, v2) = read_vertex(vertices, uvs, tri[1], stride);
            let (x3, y3, u3, v3) = read_vertex(vertices, uvs, tri[2], stride);

            for clip in polygons {
                let s = clipped_vertices.len();
                let clipped = clip_triangle(
                    &mut self.clip_output,
                    &mut self.scratch,
                    &mut self.scratch2,
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                    clip,
                );

                if clipped != 0 {
                    if self.clip_output.is_empty() {
                        continue;
                    }

                    let d0 = y2 - y3;
                    let d1 = x3 - x2;
                    let d2 = x1 - x3;
                    let d4 = y3 - y1;
                    let d = 1.0 / (d0 * d2 + d1 * (y1 - y3));

                    let clip_output_count = self.clip_output.len() / 2;
                    clipped_vertices.resize(s + clip_output_count * 2, 0.0);
                    clipped_uvs.resize(s + clip_output_count * 2, 0.0);

                    let mut write = s;
                    for xy in self.clip_output.chunks_exact(2) {
                        let x = xy[0];
                        let y = xy[1];
                        clipped_vertices[write] = x;
                        clipped_vertices[write + 1] = y;

                        let c0 = x - x3;
                        let c1 = y - y3;
                        let a = (d0 * c0 + d1 * c1) * d;
                        let b = (d4 * c0 + d2 * c1) * d;
                        let c = 1.0 - a - b;
                        clipped_uvs[write] = u1 * a + u2 * b + u3 * c;
                        clipped_uvs[write + 1] = v1 * a + v2 * b + v3 * c;

                        write += 2;
                    }

                    let clip_output_count_minus1 = clip_output_count - 1;
                    clipped_triangles.reserve(3 * clip_output_count_minus1.saturating_sub(1));
                    for ii in 1..clip_output_count_minus1 {
                        clipped_triangles.push(index);
                        clipped_triangles.push(index + ii as u16);
                        clipped_triangles.push(index + ii as u16 + 1);
                    }
                    index = index.wrapping_add(clip_output_count_minus1 as u16 + 1);
                } else {
                    clipped_vertices.resize(s + 6, 0.0);
                    clipped_uvs.resize(s + 6, 0.0);

                    clipped_vertices[s] = x1;
                    clipped_vertices[s + 1] = y1;
                    clipped_vertices[s + 2] = x2;
                    clipped_vertices[s + 3] = y2;
                    clipped_vertices[s + 4] = x3;
                    clipped_vertices[s + 5] = y3;

                    clipped_uvs[s] = u1;
                    clipped_uvs[s + 1] = v1;
                    clipped_uvs[s + 2] = u2;
                    clipped_uvs[s + 3] = v2;
                    clipped_uvs[s + 4] = u3;
                    clipped_uvs[s + 5] = v3;

                    clipped_triangles.push(index);
                    clipped_triangles.push(index + 1);
                    clipped_triangles.push(index + 2);
                    index = index.wrapping_add(3);
                    continue 'outer;
                }
            }
        }

        (clipped_vertices, clipped_uvs, clipped_triangles)
    }

    #[allow(clippy::too_many_arguments)]
    fn clip_inverse(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        polygon: &[f32],
    ) {
        self.inverse_vertices.clear();
        if polygon.len() < 4 {
            return;
        }

        let last = polygon.len() - 4;
        let mut input = vec![x1, y1, x2, y2, x3, y3, x1, y1];
        let mut output = Vec::<f32>::new();

        let mut i = 0usize;
        loop {
            let edge_x = polygon[i];
            let edge_y = polygon[i + 1];
            let ex = edge_x - polygon[i + 2];
            let ey = edge_y - polygon[i + 3];
            let output_start = output.len();
            let fragment_start = self.inverse_vertices.len();
            self.inverse_vertices.push(0.0);

            let mut ii = 0usize;
            while ii + 3 < input.len() {
                let x1 = input[ii];
                let y1 = input[ii + 1];
                ii += 2;
                let x2 = input[ii];
                let y2 = input[ii + 1];

                let s2 = ey * (edge_x - x2) > ex * (edge_y - y2);
                let s1 = ey * (edge_x - x1) - ex * (edge_y - y1);

                if s1 > 0.0 {
                    if s2 {
                        output.push(x2);
                        output.push(y2);
                    } else {
                        let ix = x2 - x1;
                        let iy = y2 - y1;
                        let t = s1 / (ix * ey - iy * ex);
                        if (0.0..=1.0).contains(&t) {
                            let cx = x1 + ix * t;
                            let cy = y1 + iy * t;
                            output.push(cx);
                            output.push(cy);
                            self.inverse_vertices.push(cx);
                            self.inverse_vertices.push(cy);
                            self.inverse_vertices.push(x2);
                            self.inverse_vertices.push(y2);
                        } else {
                            output.push(x2);
                            output.push(y2);
                        }
                    }
                } else if s2 {
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let t = s1 / (dx * ey - dy * ex);
                    if (0.0..=1.0).contains(&t) {
                        let cx = x1 + dx * t;
                        let cy = y1 + dy * t;
                        self.inverse_vertices.push(cx);
                        self.inverse_vertices.push(cy);
                        output.push(cx);
                        output.push(cy);
                        output.push(x2);
                        output.push(y2);
                    } else {
                        output.push(x2);
                        output.push(y2);
                    }
                } else {
                    self.inverse_vertices.push(x2);
                    self.inverse_vertices.push(y2);
                }
            }

            let fragment_size = self.inverse_vertices.len() - fragment_start - 1;
            if fragment_size >= 6 {
                self.inverse_vertices[fragment_start] = fragment_size as f32;
            } else {
                self.inverse_vertices.truncate(fragment_start);
            }

            if output_start == output.len() {
                break;
            }

            output.push(output[0]);
            output.push(output[1]);

            if i == last {
                break;
            }

            std::mem::swap(&mut input, &mut output);
            output.clear();
            i += 2;
        }
    }
}

fn read_vertex(vertices: &[f32], uvs: &[f32], vertex: u16, stride: usize) -> (f32, f32, f32, f32) {
    let offset = vertex as usize * stride;
    (
        vertices[offset],
        vertices[offset + 1],
        uvs[offset],
        uvs[offset + 1],
    )
}

fn make_clockwise(polygon: &mut [f32]) -> bool {
    if polygon.len() < 6 {
        return false;
    }
    let vertices_length = polygon.len();

    let mut no_cw = true;
    let mut no_ccw = true;
    let mut area = 0.0;
    let mut prev_x = polygon[vertices_length - 2];
    let mut prev_y = polygon[vertices_length - 1];
    let mut curr_x = polygon[0];
    let mut curr_y = polygon[1];

    let mut i = 2usize;
    while i < vertices_length {
        let next_x = polygon[i];
        let next_y = polygon[i + 1];
        area += curr_x * next_y - next_x * curr_y;
        let cross = (curr_x - prev_x) * (next_y - curr_y) - (curr_y - prev_y) * (next_x - curr_x);
        no_ccw &= cross <= 0.0;
        no_cw &= cross >= 0.0;
        prev_x = curr_x;
        prev_y = curr_y;
        curr_x = next_x;
        curr_y = next_y;
        i += 2;
    }

    area += curr_x * polygon[1] - polygon[0] * curr_y;
    let cross =
        (curr_x - prev_x) * (polygon[1] - curr_y) - (curr_y - prev_y) * (polygon[0] - curr_x);
    no_ccw &= cross <= 0.0;
    no_cw &= cross >= 0.0;

    if area >= 0.0 {
        let last_x = vertices_length - 2;
        let half = vertices_length / 2;
        let mut i = 0usize;
        while i < half {
            let other = last_x - i;
            polygon.swap(i, other);
            polygon.swap(i + 1, other + 1);
            i += 2;
        }
        no_cw
    } else {
        no_ccw
    }
}

fn make_convex(polygon: &mut Vec<f32>) {
    let n = polygon.len();
    if n < 6 {
        return;
    }

    let mut sorted = polygon.clone();
    let mut i = 2usize;
    while i < n {
        let x = sorted[i];
        let y = sorted[i + 1];
        let mut p = i as isize - 2;
        while p >= 0 {
            let pi = p as usize;
            if sorted[pi] < x || (sorted[pi] == x && sorted[pi + 1] <= y) {
                break;
            }
            sorted[pi + 2] = sorted[pi];
            sorted[pi + 3] = sorted[pi + 1];
            p -= 2;
        }
        let target = (p + 2) as usize;
        sorted[target] = x;
        sorted[target + 1] = y;
        i += 2;
    }

    polygon.resize(n + 2, 0.0);
    polygon[0] = sorted[0];
    polygon[1] = sorted[1];
    polygon[2] = sorted[2];
    polygon[3] = sorted[3];

    let mut s = 4usize;
    let mut i = 4usize;
    while i < n {
        let x = sorted[i];
        let y = sorted[i + 1];
        while (polygon[s - 2] - polygon[s - 4]) * (y - polygon[s - 3])
            - (polygon[s - 1] - polygon[s - 3]) * (x - polygon[s - 4])
            >= 0.0
        {
            s -= 2;
            if s == 2 {
                break;
            }
        }
        polygon[s] = x;
        polygon[s + 1] = y;
        s += 2;
        i += 2;
    }

    polygon[s] = sorted[n - 4];
    polygon[s + 1] = sorted[n - 3];
    let t = s;
    s += 2;

    let mut i = n as isize - 6;
    while i >= 0 {
        let x = sorted[i as usize];
        let y = sorted[i as usize + 1];
        while (polygon[s - 2] - polygon[s - 4]) * (y - polygon[s - 3])
            - (polygon[s - 1] - polygon[s - 3]) * (x - polygon[s - 4])
            >= 0.0
        {
            s -= 2;
            if s == t {
                break;
            }
        }
        polygon[s] = x;
        polygon[s + 1] = y;
        s += 2;
        i -= 2;
    }

    polygon.truncate(s - 2);
}

#[allow(clippy::too_many_arguments)]
fn clip_triangle(
    out: &mut Vec<f32>,
    scratch: &mut Vec<f32>,
    _scratch2: &mut Vec<f32>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    clipping_area: &[f32],
) -> i32 {
    let mut clipped: i32 = 0;

    let mut input_is_original_output = clipping_area.len() % 4 >= 2;
    let mut output_is_original_output = !input_is_original_output;
    let (mut input, mut output): (&mut Vec<f32>, &mut Vec<f32>) = if input_is_original_output {
        (out, scratch)
    } else {
        (scratch, out)
    };

    input.clear();
    input.extend_from_slice(&[x1, y1, x2, y2, x3, y3, x1, y1]);
    output.clear();

    let clipping_vertices_last = clipping_area.len() - 4;
    let mut i = 0usize;
    loop {
        let edge_x = clipping_area[i];
        let edge_y = clipping_area[i + 1];
        let ex = edge_x - clipping_area[i + 2];
        let ey = edge_y - clipping_area[i + 3];

        let output_start = output.len();

        let mut ii = 0usize;
        while ii + 3 < input.len() {
            let input_x = input[ii];
            let input_y = input[ii + 1];
            ii += 2;
            let input_x2 = input[ii];
            let input_y2 = input[ii + 1];

            let s2 = ey * (edge_x - input_x2) > ex * (edge_y - input_y2);
            let s1 = ey * (edge_x - input_x) - ex * (edge_y - input_y);

            if s1 > 0.0 {
                if s2 {
                    output.push(input_x2);
                    output.push(input_y2);
                    continue;
                }

                let ix = input_x2 - input_x;
                let iy = input_y2 - input_y;
                let t = s1 / (ix * ey - iy * ex);
                if (0.0..=1.0).contains(&t) {
                    output.push(input_x + ix * t);
                    output.push(input_y + iy * t);
                } else {
                    output.push(input_x2);
                    output.push(input_y2);
                }
            } else if s2 {
                let ix = input_x2 - input_x;
                let iy = input_y2 - input_y;
                let t = s1 / (ix * ey - iy * ex);
                if (0.0..=1.0).contains(&t) {
                    output.push(input_x + ix * t);
                    output.push(input_y + iy * t);
                    output.push(input_x2);
                    output.push(input_y2);
                } else {
                    output.push(input_x2);
                    output.push(input_y2);
                    continue;
                }
            }

            clipped = -1;
        }

        if output_start == output.len() {
            if output_is_original_output {
                output.clear();
            } else {
                input.clear();
            }
            return 1;
        }

        output.push(output[0]);
        output.push(output[1]);

        if i == clipping_vertices_last {
            break;
        }

        std::mem::swap(&mut input, &mut output);
        std::mem::swap(
            &mut input_is_original_output,
            &mut output_is_original_output,
        );
        output.clear();

        i += 2;
    }

    let output_len = output.len().saturating_sub(2);
    if output_is_original_output {
        output.truncate(output_len);
    } else {
        input.clear();
        input.extend_from_slice(&output[..output_len]);
    }

    clipped
}
