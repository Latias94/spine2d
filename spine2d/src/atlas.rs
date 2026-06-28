use crate::Error;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Atlas {
    pages: Vec<AtlasPage>,
    regions: Vec<AtlasRegion>,
}

impl Atlas {
    pub fn flip_v(&mut self) {
        for region in &mut self.regions {
            region.v = 1.0 - region.v;
            region.v2 = 1.0 - region.v2;
        }
    }

    pub fn find_region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.iter().find(|region| region.get_name() == name)
    }

    pub fn get_pages(&self) -> &[AtlasPage] {
        &self.pages
    }

    pub fn get_regions(&self) -> &[AtlasRegion] {
        &self.regions
    }
}

#[derive(Clone, Debug)]
pub struct AtlasPage {
    pub name: String,
    pub texture_path: String,
    pub index: usize,
    pub format: AtlasFormat,
    pub width: u32,
    pub height: u32,
    pub pma: bool,
    pub min_filter: AtlasFilter,
    pub mag_filter: AtlasFilter,
    pub wrap_u: AtlasWrap,
    pub wrap_v: AtlasWrap,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum AtlasFilter {
    Unknown,
    #[default]
    Nearest,
    Linear,
    MipMap,
    MipMapNearestNearest,
    MipMapNearestLinear,
    MipMapLinearNearest,
    MipMapLinearLinear,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum AtlasFormat {
    Alpha,
    Intensity,
    LuminanceAlpha,
    RGB565,
    RGBA4444,
    RGB888,
    #[default]
    RGBA8888,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum AtlasWrap {
    #[default]
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

#[derive(Clone, Debug)]
pub struct AtlasRegion {
    name: String,
    page: usize,
    index: i32,
    rotate: bool,
    degrees: i32,
    x: u32,
    y: u32,
    packed_width: u32,
    packed_height: u32,
    offset_x: f32,
    offset_y: f32,
    original_width: u32,
    original_height: u32,
    splits: Vec<i32>,
    pads: Vec<i32>,
    names: Vec<String>,
    values: Vec<f32>,
    u: f32,
    v: f32,
    u2: f32,
    v2: f32,
    region_width: u32,
    region_height: u32,
}

impl AtlasRegion {
    pub fn get_page(&self) -> usize {
        self.page
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }

    pub fn get_x(&self) -> u32 {
        self.x
    }

    pub fn get_y(&self) -> u32 {
        self.y
    }

    pub fn get_offset_x(&self) -> f32 {
        self.offset_x
    }

    pub fn get_offset_y(&self) -> f32 {
        self.offset_y
    }

    pub fn get_packed_width(&self) -> u32 {
        self.packed_width
    }

    pub fn get_packed_height(&self) -> u32 {
        self.packed_height
    }

    pub fn get_original_width(&self) -> u32 {
        self.original_width
    }

    pub fn get_original_height(&self) -> u32 {
        self.original_height
    }

    pub fn get_rotate(&self) -> bool {
        self.rotate
    }

    pub fn get_degrees(&self) -> i32 {
        self.degrees
    }

    pub fn get_splits(&self) -> &[i32] {
        &self.splits
    }

    pub fn get_pads(&self) -> &[i32] {
        &self.pads
    }

    pub fn get_names(&self) -> &[String] {
        &self.names
    }

    pub fn get_values(&self) -> &[f32] {
        &self.values
    }

    pub fn get_u(&self) -> f32 {
        self.u
    }

    pub fn get_v(&self) -> f32 {
        self.v
    }

    pub fn get_u2(&self) -> f32 {
        self.u2
    }

    pub fn get_v2(&self) -> f32 {
        self.v2
    }

    pub fn get_region_width(&self) -> u32 {
        self.region_width
    }

    pub fn get_region_height(&self) -> u32 {
        self.region_height
    }
}

impl FromStr for Atlas {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_atlas(s)
    }
}

fn parse_atlas(input: &str) -> Result<Atlas, Error> {
    let mut pages = Vec::new();
    let mut regions = Vec::new();

    fn finalize_region(mut region: AtlasRegion, page: &AtlasPage) -> AtlasRegion {
        if region.original_width == 0 && region.original_height == 0 {
            region.original_width = region.packed_width;
            region.original_height = region.packed_height;
        }
        let page_width = page.width.max(1) as f32;
        let page_height = page.height.max(1) as f32;

        region.u = region.x as f32 / page_width;
        region.v = region.y as f32 / page_height;
        if region.degrees == 90 {
            region.u2 = (region.x + region.packed_height) as f32 / page_width;
            region.v2 = (region.y + region.packed_width) as f32 / page_height;
        } else {
            region.u2 = (region.x + region.packed_width) as f32 / page_width;
            region.v2 = (region.y + region.packed_height) as f32 / page_height;
        }
        region.region_width = ((region.u2 - region.u) * page_width).abs() as u32;
        region.region_height = ((region.v2 - region.v) * page_height).abs() as u32;
        if region.degrees == 90 {
            std::mem::swap(&mut region.packed_width, &mut region.packed_height);
        }
        region
    }

    let lines = input.lines().map(str::trim).collect::<Vec<_>>();
    let mut cursor = 0;

    while cursor < lines.len() && lines[cursor].is_empty() {
        cursor += 1;
    }

    while cursor < lines.len() && parse_entry(lines[cursor]).is_some() {
        cursor += 1;
    }

    let mut current_page: Option<usize> = None;

    while cursor < lines.len() {
        let line = lines[cursor];
        if line.is_empty() {
            current_page = None;
            cursor += 1;
            continue;
        }

        if let Some(page_index) = current_page {
            let mut region = AtlasRegion {
                name: line.to_string(),
                page: page_index,
                index: 0,
                rotate: false,
                degrees: 0,
                x: 0,
                y: 0,
                packed_width: 0,
                packed_height: 0,
                offset_x: 0.0,
                offset_y: 0.0,
                original_width: 0,
                original_height: 0,
                splits: Vec::new(),
                pads: Vec::new(),
                names: Vec::new(),
                values: Vec::new(),
                u: 0.0,
                v: 0.0,
                u2: 0.0,
                v2: 0.0,
                region_width: 0,
                region_height: 0,
            };
            cursor += 1;

            while cursor < lines.len() {
                let Some((key, value)) = parse_entry(lines[cursor]) else {
                    break;
                };

                match key {
                    "rotate" => {
                        let degrees = parse_degrees(value);
                        region.rotate = degrees == 90;
                        region.degrees = degrees;
                    }
                    "bounds" => {
                        let (x, y, w, h) =
                            parse_quad_u32(value).ok_or_else(|| Error::AtlasParse {
                                message: format!("invalid region bounds: {value}"),
                            })?;
                        region.x = x;
                        region.y = y;
                        region.packed_width = w;
                        region.packed_height = h;
                    }
                    "xy" => {
                        let (x, y) = parse_pair_u32(value).ok_or_else(|| Error::AtlasParse {
                            message: format!("invalid region xy: {value}"),
                        })?;
                        region.x = x;
                        region.y = y;
                    }
                    "size" => {
                        let (w, h) = parse_pair_u32(value).ok_or_else(|| Error::AtlasParse {
                            message: format!("invalid region size: {value}"),
                        })?;
                        region.packed_width = w;
                        region.packed_height = h;
                    }
                    "orig" => {
                        let (w, h) = parse_pair_u32(value).ok_or_else(|| Error::AtlasParse {
                            message: format!("invalid region orig: {value}"),
                        })?;
                        region.original_width = w;
                        region.original_height = h;
                    }
                    "offset" => {
                        let (x, y) = parse_pair_i32(value).ok_or_else(|| Error::AtlasParse {
                            message: format!("invalid region offset: {value}"),
                        })?;
                        region.offset_x = x as f32;
                        region.offset_y = y as f32;
                    }
                    "offsets" => {
                        let (x, y, w, h) =
                            parse_quad_i32_u32(value).ok_or_else(|| Error::AtlasParse {
                                message: format!("invalid region offsets: {value}"),
                            })?;
                        region.offset_x = x as f32;
                        region.offset_y = y as f32;
                        region.original_width = w;
                        region.original_height = h;
                    }
                    "index" => {
                        region.index = value.parse().unwrap_or(0);
                    }
                    _ => {
                        region.names.push(key.to_string());
                        region.values.extend(parse_region_values(value));
                    }
                }

                cursor += 1;
            }

            let page = &pages[page_index];
            regions.push(finalize_region(region, page));
        } else {
            let page_index = pages.len();
            pages.push(AtlasPage {
                name: line.to_string(),
                texture_path: line.to_string(),
                index: page_index,
                format: AtlasFormat::default(),
                width: 0,
                height: 0,
                pma: false,
                min_filter: AtlasFilter::default(),
                mag_filter: AtlasFilter::default(),
                wrap_u: AtlasWrap::default(),
                wrap_v: AtlasWrap::default(),
            });
            current_page = Some(page_index);
            cursor += 1;

            while cursor < lines.len() {
                let Some((key, value)) = parse_entry(lines[cursor]) else {
                    break;
                };

                match key {
                    "format" => {
                        if let Some(page) = pages.get_mut(page_index) {
                            page.format = parse_format(value);
                        }
                    }
                    "size" => {
                        let (w, h) = parse_pair_u32(value).ok_or_else(|| Error::AtlasParse {
                            message: format!("invalid page size: {value}"),
                        })?;
                        if let Some(page) = pages.get_mut(page_index) {
                            page.width = w;
                            page.height = h;
                        }
                    }
                    "filter" => {
                        let (min, mag) = parse_pair_str(value)
                            .map(|(a, b)| (parse_filter(a), parse_filter(b)))
                            .unwrap_or_else(|| {
                                let f = parse_filter(value);
                                (f.clone(), f)
                            });
                        if let Some(page) = pages.get_mut(page_index) {
                            page.min_filter = min;
                            page.mag_filter = mag;
                        }
                    }
                    "repeat" => {
                        let (wrap_u, wrap_v) = parse_repeat(value);
                        if let Some(page) = pages.get_mut(page_index) {
                            page.wrap_u = wrap_u;
                            page.wrap_v = wrap_v;
                        }
                    }
                    "pma" => {
                        if let Some(page) = pages.get_mut(page_index) {
                            page.pma = matches!(value, "true");
                        }
                    }
                    _ => {}
                }

                cursor += 1;
            }
        }
    }

    if pages.is_empty() {
        return Err(Error::AtlasParse {
            message: "empty atlas".to_string(),
        });
    }

    Ok(Atlas { pages, regions })
}

fn parse_entry(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let (key, value) = line.split_once(':')?;
    Some((key.trim(), value.trim()))
}

fn parse_pair_u32(value: &str) -> Option<(u32, u32)> {
    let (a, b) = value.split_once(',')?;
    let a = a.trim().parse().ok()?;
    let b = b.trim().parse().ok()?;
    Some((a, b))
}

fn parse_pair_str(value: &str) -> Option<(&str, &str)> {
    let (a, b) = value.split_once(',')?;
    Some((a.trim(), b.trim()))
}

fn parse_region_values(value: &str) -> Vec<f32> {
    value
        .split(',')
        .take(4)
        .map(|part| part.trim().parse::<i32>().unwrap_or(0) as f32)
        .collect()
}

fn parse_format(value: &str) -> AtlasFormat {
    match value {
        "Alpha" => AtlasFormat::Alpha,
        "Intensity" => AtlasFormat::Intensity,
        "LuminanceAlpha" => AtlasFormat::LuminanceAlpha,
        "RGB565" => AtlasFormat::RGB565,
        "RGBA4444" => AtlasFormat::RGBA4444,
        "RGB888" => AtlasFormat::RGB888,
        "RGBA8888" => AtlasFormat::RGBA8888,
        _ => AtlasFormat::RGBA8888,
    }
}

fn parse_quad_u32(value: &str) -> Option<(u32, u32, u32, u32)> {
    let mut it = value.split(',').map(|s| s.trim().parse::<u32>().ok());
    let a = it.next().flatten()?;
    let b = it.next().flatten()?;
    let c = it.next().flatten()?;
    let d = it.next().flatten()?;
    Some((a, b, c, d))
}

fn parse_pair_i32(value: &str) -> Option<(i32, i32)> {
    let (a, b) = value.split_once(',')?;
    let a = a.trim().parse().ok()?;
    let b = b.trim().parse().ok()?;
    Some((a, b))
}

fn parse_quad_i32_u32(value: &str) -> Option<(i32, i32, u32, u32)> {
    let mut it = value.split(',').map(|s| s.trim());
    let x: i32 = it.next()?.parse().ok()?;
    let y: i32 = it.next()?.parse().ok()?;
    let w: u32 = it.next()?.parse().ok()?;
    let h: u32 = it.next()?.parse().ok()?;
    Some((x, y, w, h))
}

fn parse_degrees(value: &str) -> i32 {
    match value {
        "true" => 90,
        "false" => 0,
        _ => value.parse::<i32>().unwrap_or(0),
    }
}

fn parse_filter(value: &str) -> AtlasFilter {
    match value {
        "Nearest" => AtlasFilter::Nearest,
        "Linear" => AtlasFilter::Linear,
        "MipMap" => AtlasFilter::MipMap,
        "MipMapNearestNearest" => AtlasFilter::MipMapNearestNearest,
        "MipMapNearestLinear" => AtlasFilter::MipMapNearestLinear,
        "MipMapLinearNearest" => AtlasFilter::MipMapLinearNearest,
        "MipMapLinearLinear" => AtlasFilter::MipMapLinearLinear,
        _ => AtlasFilter::Unknown,
    }
}

fn parse_repeat(value: &str) -> (AtlasWrap, AtlasWrap) {
    let wrap_u = if value.contains('x') {
        AtlasWrap::Repeat
    } else {
        AtlasWrap::ClampToEdge
    };
    let wrap_v = if value.contains('y') {
        AtlasWrap::Repeat
    } else {
        AtlasWrap::ClampToEdge
    };
    (wrap_u, wrap_v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_atlas_one_page_one_region() {
        let atlas = r#"
page.png
size: 64,64
scale: 0.5
pma: true
filter: Linear, Linear
head
  rotate: false
  xy: 0, 0
  size: 16, 8
"#
        .parse::<Atlas>()
        .unwrap();

        assert_eq!(atlas.get_pages().len(), 1);
        assert_eq!(atlas.get_pages()[0].name, "page.png");
        assert_eq!(atlas.get_pages()[0].texture_path, "page.png");
        assert_eq!(atlas.get_pages()[0].format, AtlasFormat::RGBA8888);
        assert_eq!(atlas.get_pages()[0].width, 64);
        assert_eq!(atlas.get_pages()[0].height, 64);
        assert!(atlas.get_pages()[0].pma);
        assert_eq!(atlas.get_pages()[0].min_filter, AtlasFilter::Linear);
        assert_eq!(atlas.get_pages()[0].mag_filter, AtlasFilter::Linear);
        assert_eq!(atlas.get_pages()[0].wrap_u, AtlasWrap::ClampToEdge);
        assert_eq!(atlas.get_pages()[0].wrap_v, AtlasWrap::ClampToEdge);

        let region = atlas.find_region("head").unwrap();
        assert_eq!(region.get_page(), 0);
        assert_eq!(region.get_degrees(), 0);
        assert_eq!(region.get_x(), 0);
        assert_eq!(region.get_y(), 0);
        assert_eq!(region.get_packed_width(), 16);
        assert_eq!(region.get_packed_height(), 8);
    }

    #[test]
    fn parse_atlas_multiple_pages_assigns_region_pages() {
        let atlas = r#"
page0.png
size: 32,32
r0
  bounds: 0, 0, 1, 1

page1.png
size: 64,64
r1
  bounds: 2, 3, 4, 5
"#
        .parse::<Atlas>()
        .unwrap();

        assert_eq!(atlas.get_pages().len(), 2);
        assert_eq!(atlas.get_pages()[0].name, "page0.png");
        assert_eq!(atlas.get_pages()[0].texture_path, "page0.png");
        assert_eq!(atlas.get_pages()[0].index, 0);
        assert_eq!(atlas.get_pages()[1].name, "page1.png");
        assert_eq!(atlas.get_pages()[1].texture_path, "page1.png");
        assert_eq!(atlas.get_pages()[1].index, 1);

        let r0 = atlas.find_region("r0").unwrap();
        let r1 = atlas.find_region("r1").unwrap();
        assert_eq!(r0.get_page(), 0);
        assert_eq!(r1.get_page(), 1);
        assert_eq!(r1.get_x(), 2);
        assert_eq!(r1.get_y(), 3);
        assert_eq!(r1.get_packed_width(), 4);
        assert_eq!(r1.get_packed_height(), 5);
    }

    #[test]
    fn parse_atlas_regions_keep_cpp_array_order_index_and_extra_values() {
        let atlas = r#"
page.png
size: 64,64
beta
  bounds: 0, 0, 10, 11
  index: 3
  split: 1, 2, 3, 4
alpha
  bounds: 1, 2, 12, 13
  index: -1
  pad: 5, 6, 7, 8
"#
        .parse::<Atlas>()
        .unwrap();

        assert_eq!(
            atlas
                .get_regions()
                .iter()
                .map(AtlasRegion::get_name)
                .collect::<Vec<_>>(),
            vec!["beta", "alpha"]
        );

        let beta = atlas.find_region("beta").unwrap();
        assert_eq!(beta.get_index(), 3);
        assert!(beta.get_splits().is_empty());
        assert_eq!(beta.get_names(), ["split".to_string()]);
        assert_eq!(beta.get_values(), [1.0, 2.0, 3.0, 4.0]);

        let alpha = atlas.find_region("alpha").unwrap();
        assert_eq!(alpha.get_index(), -1);
        assert!(alpha.get_pads().is_empty());
        assert_eq!(alpha.get_names(), ["pad".to_string()]);
        assert_eq!(alpha.get_values(), [5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn parse_atlas_region_computes_texture_region_uvs_and_flip_v() {
        let mut atlas = r#"
page.png
size: 64,64
head
  rotate: true
  xy: 16, 32
  size: 16, 8
"#
        .parse::<Atlas>()
        .unwrap();

        let region = atlas.find_region("head").unwrap();
        assert_eq!(region.get_region_width(), 8);
        assert_eq!(region.get_region_height(), 16);
        assert_eq!(region.get_packed_width(), 8);
        assert_eq!(region.get_packed_height(), 16);
        assert!((region.get_u() - 16.0 / 64.0).abs() <= 1.0e-6);
        assert!((region.get_v() - 32.0 / 64.0).abs() <= 1.0e-6);
        assert!((region.get_u2() - 24.0 / 64.0).abs() <= 1.0e-6);
        assert!((region.get_v2() - 48.0 / 64.0).abs() <= 1.0e-6);

        atlas.flip_v();
        let region = atlas.find_region("head").unwrap();
        assert!((region.get_v() - (1.0 - 32.0 / 64.0)).abs() <= 1.0e-6);
        assert!((region.get_v2() - (1.0 - 48.0 / 64.0)).abs() <= 1.0e-6);
    }

    #[test]
    fn parse_atlas_region_bounds_sets_xy_and_size() {
        let atlas = r#"
page.png
size: 64,64
head
  bounds: 16, 32, 8, 4
"#
        .parse::<Atlas>()
        .unwrap();

        let region = atlas.find_region("head").unwrap();
        assert_eq!(region.get_x(), 16);
        assert_eq!(region.get_y(), 32);
        assert_eq!(region.get_packed_width(), 8);
        assert_eq!(region.get_packed_height(), 4);
        assert_eq!(region.get_original_width(), 8);
        assert_eq!(region.get_original_height(), 4);
    }

    #[test]
    fn parse_atlas_page_filter_and_repeat() {
        let atlas = r#"
page.png
format: RGB888
size: 64,64
filter: Nearest, Linear
repeat: xy
head
  bounds: 0, 0, 1, 1
"#
        .parse::<Atlas>()
        .unwrap();

        let page = &atlas.get_pages()[0];
        assert_eq!(page.format, AtlasFormat::RGB888);
        assert_eq!(page.min_filter, AtlasFilter::Nearest);
        assert_eq!(page.mag_filter, AtlasFilter::Linear);
        assert_eq!(page.wrap_u, AtlasWrap::Repeat);
        assert_eq!(page.wrap_v, AtlasWrap::Repeat);
    }

    #[test]
    fn parse_atlas_unknown_filter_matches_cpp_unknown() {
        let atlas = r#"
page.png
filter: Strange, Linear
head
  bounds: 0, 0, 1, 1
"#
        .parse::<Atlas>()
        .unwrap();

        let page = &atlas.get_pages()[0];
        assert_eq!(page.min_filter, AtlasFilter::Unknown);
        assert_eq!(page.mag_filter, AtlasFilter::Linear);
    }

    #[test]
    fn parse_atlas_region_orig_and_offset() {
        let atlas = r#"
page.png
size: 64,64
head
  xy: 0, 0
  size: 10, 11
  orig: 20, 21
  offset: 3, 4
"#
        .parse::<Atlas>()
        .unwrap();

        let region = atlas.find_region("head").unwrap();
        assert_eq!(region.get_packed_width(), 10);
        assert_eq!(region.get_packed_height(), 11);
        assert_eq!(region.get_original_width(), 20);
        assert_eq!(region.get_original_height(), 21);
        assert_eq!(region.get_offset_x(), 3.0);
        assert_eq!(region.get_offset_y(), 4.0);
    }

    #[test]
    fn parse_atlas_region_offsets_compact_field() {
        let atlas = r#"
page.png
size: 64,64
head
  bounds: 1, 2, 3, 4
  offsets: 5, 6, 7, 8
"#
        .parse::<Atlas>()
        .unwrap();

        let region = atlas.find_region("head").unwrap();
        assert_eq!(region.get_x(), 1);
        assert_eq!(region.get_y(), 2);
        assert_eq!(region.get_packed_width(), 3);
        assert_eq!(region.get_packed_height(), 4);
        assert_eq!(region.get_offset_x(), 5.0);
        assert_eq!(region.get_offset_y(), 6.0);
        assert_eq!(region.get_original_width(), 7);
        assert_eq!(region.get_original_height(), 8);
    }

    #[test]
    fn parse_atlas_region_rotate_degrees_accepts_true_false_and_numbers() {
        let atlas = r#"
page.png
size: 64,64
r0
  bounds: 0, 0, 1, 1
  rotate: false
r90
  bounds: 0, 0, 1, 1
  rotate: true
r180
  bounds: 0, 0, 1, 1
  rotate: 180
r270
  bounds: 0, 0, 1, 1
  rotate: 270
"#
        .parse::<Atlas>()
        .unwrap();

        assert_eq!(atlas.find_region("r0").unwrap().get_degrees(), 0);
        assert_eq!(atlas.find_region("r90").unwrap().get_degrees(), 90);
        assert!(atlas.find_region("r90").unwrap().get_rotate());
        assert_eq!(atlas.find_region("r180").unwrap().get_degrees(), 180);
        assert!(!atlas.find_region("r180").unwrap().get_rotate());
        assert_eq!(atlas.find_region("r270").unwrap().get_degrees(), 270);
        assert!(!atlas.find_region("r270").unwrap().get_rotate());
    }
}
