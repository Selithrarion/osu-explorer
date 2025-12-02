use super::model::{Beatmap, HitObject, HitObjectType};
use anyhow::anyhow;
use std::path::Path;

pub fn parse_beatmap_from_file(path: &Path) -> Result<Beatmap, anyhow::Error> {
    let contents = std::fs::read_to_string(path)?;
    let parsed_map = osuparse::parse_beatmap(&contents)
        .map_err(|e| anyhow!("Failed to parse beatmap with osuparse: {:?}", e))?;

    let beatmap = Beatmap {
        title: parsed_map.metadata.title.clone(),
        artist: parsed_map.metadata.artist.clone(),
        difficulty_name: parsed_map.metadata.version.clone(),
        beatmap_id: parsed_map.metadata.beatmap_id,
        beatmapset_id: parsed_map.metadata.beatmap_set_id,
        hit_objects: parsed_map
            .hit_objects
            .iter()
            .map(|ho| {
                let (x, y, start_time, _end_time, obj_type, curve_points) = match ho {
                    osuparse::HitObject::HitCircle(obj) => (
                        obj.x,
                        obj.y,
                        obj.time,
                        obj.time,
                        HitObjectType::Circle,
                        None,
                    ),
                    osuparse::HitObject::Slider(obj) => {
                        let points = obj
                            .curve_points
                            .iter()
                            .map(|p| (p.0 as f32, p.1 as f32))
                            .collect();
                        (
                            obj.x,
                            obj.y,
                            obj.time,
                            calculate_slider_end_time(&parsed_map, obj),
                            HitObjectType::Slider,
                            Some(points),
                        )
                    }
                    osuparse::HitObject::Spinner(obj) => (
                        256,
                        192,
                        obj.time,
                        obj.end_time,
                        HitObjectType::Spinner,
                        None,
                    ),
                    osuparse::HitObject::HoldNote(obj) => (
                        obj.x,
                        obj.y,
                        obj.time,
                        obj.end_time,
                        HitObjectType::HoldNote,
                        None,
                    ),
                };
                HitObject {
                    x: x as f32,
                    y: y as f32,
                    start_time: start_time as f32,
                    obj_type,
                    curve_points,
                }
            })
            .collect(),
    };

    Ok(beatmap)
}

fn calculate_slider_end_time(parsed_map: &osuparse::Beatmap, slider: &osuparse::Slider) -> i32 {
    let slider_start_time = slider.time as f32;

    // find red
    let mut base_ms_per_beat = 1000.0;
    for tp in parsed_map.timing_points.iter().rev() {
        if !tp.inherited && tp.offset <= slider_start_time {
            base_ms_per_beat = tp.ms_per_beat;
            break;
        }
    }

    // find green
    let mut speed_multiplier = 1.0;
    for tp in parsed_map.timing_points.iter().rev() {
        if tp.offset <= slider_start_time {
            if tp.inherited {
                speed_multiplier = -100.0 / tp.ms_per_beat;
            }
            break;
        }
    }

    let slider_velocity = parsed_map.difficulty.slider_multiplier as f32 * 100.0 * speed_multiplier;
    let single_pass_duration = (slider.pixel_length / slider_velocity) * base_ms_per_beat;
    let total_duration = single_pass_duration * slider.repeat as f32;

    slider.time + total_duration as i32
}
