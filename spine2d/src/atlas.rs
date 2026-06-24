use crate::Error;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Atlas {
    pub pages: Vec<AtlasPage>,
    pub regions: Vec<AtlasRegion>,
}

impl Atlas {
    pub fn parse(input: &str) -> Result<Self, Error> {
        parse_atlas(input)
    }

    pub fn region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.iter().find(|region| region.name == name)
    }

    pub fn page(&self, index: usize) -> Option<&AtlasPage> {
        self.pages.get(index)
    }
}

#[derive(Clone, Debug)]
pub struct AtlasPage {
    pub name: String,
    pub index: usize,
    pub format: AtlasFormat,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub pma: bool,
    pub min_filter: AtlasFilter,
    pub mag_filter: AtlasFilter,
    pub wrap_u: AtlasWrap,
    pub wrap_v: AtlasWrap,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum AtlasFilter {
    #[default]
    Nearest,
    Linear,
    MipMap,
    MipMapNearestNearest,
    MipMapNearestLinear,
    MipMapLinearNearest,
    MipMapLinearLinear,
    Other(String),
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
    Repeat,
}

#[derive(Clone, Debug)]
pub struct AtlasRegion {
    pub name: String,
    pub page: usize,
    pub index: i32,
    pub degrees: i32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub original_width: u32,
    pub original_height: u32,
    pub names: Vec<String>,
    pub values: Vec<f32>,
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

    fn finalize_region(mut region: AtlasRegion) -> AtlasRegion {
        if region.original_width == 0 {
            region.original_width = region.width;
        }
        if region.original_height == 0 {
            region.original_height = region.height;
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

        if current_page.is_none() {
            let page_index = pages.len();
            pages.push(AtlasPage {
                name: line.to_string(),
                index: page_index,
                format: AtlasFormat::default(),
                width: 0,
                height: 0,
                scale: 1.0,
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
                    "scale" => {
                        let s: f32 = value.parse().map_err(|_| Error::AtlasParse {
                            message: format!("invalid page scale: {value}"),
                        })?;
                        if let Some(page) = pages.get_mut(page_index) {
                            page.scale = if s.is_finite() { s } else { 1.0 };
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
        } else {
            let page_index = current_page.expect("current page exists for atlas region");
            let mut region = AtlasRegion {
                name: line.to_string(),
                page: page_index,
                index: 0,
                degrees: 0,
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                offset_x: 0,
                offset_y: 0,
                original_width: 0,
                original_height: 0,
                names: Vec::new(),
                values: Vec::new(),
            };
            cursor += 1;

            while cursor < lines.len() {
                let Some((key, value)) = parse_entry(lines[cursor]) else {
                    break;
                };

                match key {
                    "rotate" => {
                        region.degrees = parse_degrees(value);
                    }
                    "bounds" => {
                        let (x, y, w, h) =
                            parse_quad_u32(value).ok_or_else(|| Error::AtlasParse {
                                message: format!("invalid region bounds: {value}"),
                            })?;
                        region.x = x;
                        region.y = y;
                        region.width = w;
                        region.height = h;
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
                        region.width = w;
                        region.height = h;
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
                        region.offset_x = x;
                        region.offset_y = y;
                    }
                    "offsets" => {
                        let (x, y, w, h) =
                            parse_quad_i32_u32(value).ok_or_else(|| Error::AtlasParse {
                                message: format!("invalid region offsets: {value}"),
                            })?;
                        region.offset_x = x;
                        region.offset_y = y;
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

            regions.push(finalize_region(region));
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
        other => AtlasFilter::Other(other.to_string()),
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
        let atlas = Atlas::parse(
            r#"
page.png
size: 64,64
scale: 0.5
pma: true
filter: Linear, Linear
head
  rotate: false
  xy: 0, 0
  size: 16, 8
"#,
        )
        .unwrap();

        assert_eq!(atlas.pages.len(), 1);
        assert_eq!(atlas.pages[0].name, "page.png");
        assert_eq!(atlas.pages[0].format, AtlasFormat::RGBA8888);
        assert_eq!(atlas.pages[0].width, 64);
        assert_eq!(atlas.pages[0].height, 64);
        assert!((atlas.pages[0].scale - 0.5).abs() <= 1.0e-6);
        assert!(atlas.pages[0].pma);
        assert_eq!(atlas.pages[0].min_filter, AtlasFilter::Linear);
        assert_eq!(atlas.pages[0].mag_filter, AtlasFilter::Linear);
        assert_eq!(atlas.pages[0].wrap_u, AtlasWrap::ClampToEdge);
        assert_eq!(atlas.pages[0].wrap_v, AtlasWrap::ClampToEdge);

        let region = atlas.region("head").unwrap();
        assert_eq!(region.page, 0);
        assert_eq!(region.degrees, 0);
        assert_eq!(region.x, 0);
        assert_eq!(region.y, 0);
        assert_eq!(region.width, 16);
        assert_eq!(region.height, 8);
    }

    #[test]
    fn parse_atlas_multiple_pages_assigns_region_pages() {
        let atlas = Atlas::parse(
            r#"
page0.png
size: 32,32
r0
  bounds: 0, 0, 1, 1

page1.png
size: 64,64
r1
  bounds: 2, 3, 4, 5
"#,
        )
        .unwrap();

        assert_eq!(atlas.pages.len(), 2);
        assert_eq!(atlas.pages[0].name, "page0.png");
        assert_eq!(atlas.pages[0].index, 0);
        assert_eq!(atlas.pages[1].name, "page1.png");
        assert_eq!(atlas.pages[1].index, 1);

        let r0 = atlas.region("r0").unwrap();
        let r1 = atlas.region("r1").unwrap();
        assert_eq!(r0.page, 0);
        assert_eq!(r1.page, 1);
        assert_eq!(r1.x, 2);
        assert_eq!(r1.y, 3);
        assert_eq!(r1.width, 4);
        assert_eq!(r1.height, 5);
    }

    #[test]
    fn parse_atlas_regions_keep_cpp_array_order_index_and_extra_values() {
        let atlas = Atlas::parse(
            r#"
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
"#,
        )
        .unwrap();

        assert_eq!(
            atlas
                .regions
                .iter()
                .map(|region| region.name.as_str())
                .collect::<Vec<_>>(),
            vec!["beta", "alpha"]
        );

        let beta = atlas.region("beta").unwrap();
        assert_eq!(beta.index, 3);
        assert_eq!(beta.names, vec!["split"]);
        assert_eq!(beta.values, vec![1.0, 2.0, 3.0, 4.0]);

        let alpha = atlas.region("alpha").unwrap();
        assert_eq!(alpha.index, -1);
        assert_eq!(alpha.names, vec!["pad"]);
        assert_eq!(alpha.values, vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn parse_atlas_region_bounds_sets_xy_and_size() {
        let atlas = Atlas::parse(
            r#"
page.png
size: 64,64
head
  bounds: 16, 32, 8, 4
"#,
        )
        .unwrap();

        let region = atlas.region("head").unwrap();
        assert_eq!(region.x, 16);
        assert_eq!(region.y, 32);
        assert_eq!(region.width, 8);
        assert_eq!(region.height, 4);
        assert_eq!(region.original_width, 8);
        assert_eq!(region.original_height, 4);
    }

    #[test]
    fn parse_atlas_page_filter_and_repeat() {
        let atlas = Atlas::parse(
            r#"
page.png
format: RGB888
size: 64,64
filter: Nearest, Linear
repeat: xy
head
  bounds: 0, 0, 1, 1
"#,
        )
        .unwrap();

        let page = &atlas.pages[0];
        assert_eq!(page.format, AtlasFormat::RGB888);
        assert_eq!(page.min_filter, AtlasFilter::Nearest);
        assert_eq!(page.mag_filter, AtlasFilter::Linear);
        assert_eq!(page.wrap_u, AtlasWrap::Repeat);
        assert_eq!(page.wrap_v, AtlasWrap::Repeat);
    }

    #[test]
    fn parse_atlas_region_orig_and_offset() {
        let atlas = Atlas::parse(
            r#"
page.png
size: 64,64
head
  xy: 0, 0
  size: 10, 11
  orig: 20, 21
  offset: 3, 4
"#,
        )
        .unwrap();

        let region = atlas.region("head").unwrap();
        assert_eq!(region.width, 10);
        assert_eq!(region.height, 11);
        assert_eq!(region.original_width, 20);
        assert_eq!(region.original_height, 21);
        assert_eq!(region.offset_x, 3);
        assert_eq!(region.offset_y, 4);
    }

    #[test]
    fn parse_atlas_region_offsets_compact_field() {
        let atlas = Atlas::parse(
            r#"
page.png
size: 64,64
head
  bounds: 1, 2, 3, 4
  offsets: 5, 6, 7, 8
"#,
        )
        .unwrap();

        let region = atlas.region("head").unwrap();
        assert_eq!(region.x, 1);
        assert_eq!(region.y, 2);
        assert_eq!(region.width, 3);
        assert_eq!(region.height, 4);
        assert_eq!(region.offset_x, 5);
        assert_eq!(region.offset_y, 6);
        assert_eq!(region.original_width, 7);
        assert_eq!(region.original_height, 8);
    }

    #[test]
    fn parse_atlas_region_rotate_degrees_accepts_true_false_and_numbers() {
        let atlas = Atlas::parse(
            r#"
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
"#,
        )
        .unwrap();

        assert_eq!(atlas.region("r0").unwrap().degrees, 0);
        assert_eq!(atlas.region("r90").unwrap().degrees, 90);
        assert_eq!(atlas.region("r180").unwrap().degrees, 180);
        assert_eq!(atlas.region("r270").unwrap().degrees, 270);
    }
}
