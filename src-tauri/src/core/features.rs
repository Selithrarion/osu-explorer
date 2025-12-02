use super::model::{Beatmap, HitObject, HitObjectType};
use seli_vector_db::Vector;

const FEATURE_COUNT: usize = 8;

pub fn extract_features(beatmap: &Beatmap) -> Option<Vector> {
    if beatmap.hit_objects.is_empty() {
        return None;
    }

    let total_objects = beatmap.hit_objects.len() as f32;
    if total_objects < 3.0 {
        return Some(vec![0.0; FEATURE_COUNT]);
    }

    let mut stream_pattern_count = 0;
    let mut spaced_stream_pattern_count = 0;
    let mut reading_overlap_count = 0;
    let mut angle_consistency_count = 0;
    let mut grid_snap_count = 0;

    let mut flow_pattern_total = 0;
    let mut total_time_intervals = 0.0;
    let mut angle_consistency_pattern_total = 0;

    const CIRCLE_RADIUS: f32 = 36.0; // ~cs4
    const GRID_SIZE: f32 = 8.0;
    const GRID_TOLERANCE: f32 = 1.0;

    const STREAM_MAX_TIME_MS: f32 = 125.0; // >240 BPM 1/4 notes
    const DENSE_STREAM_MAX_DIST: f32 = 100.0;
    const SPACED_STREAM_MIN_DIST: f32 = 100.0;
    const SPACED_STREAM_MAX_DIST: f32 = 200.0;
    const SIMILAR_ANGLE_THRESHOLD_DEG: f32 = 15.0;

    for i in 0..beatmap.hit_objects.len() {
        let current_obj = &beatmap.hit_objects[i];

        // grid check
        if is_grid_snapped(current_obj, GRID_SIZE, GRID_TOLERANCE) {
            grid_snap_count += 1;
        }

        // flow check
        if i > 0 {
            flow_pattern_total += 1;
            let prev_obj = &beatmap.hit_objects[i - 1];
            let dist = euclidean_distance(current_obj, prev_obj);
            let time_diff = current_obj.start_time - prev_obj.start_time;
            total_time_intervals += time_diff;

            // dense stream
            if time_diff > 0.0 && time_diff < STREAM_MAX_TIME_MS && dist < DENSE_STREAM_MAX_DIST {
                stream_pattern_count += 1;
            }

            // spaced stream / alt
            if time_diff > 0.0
                && time_diff < STREAM_MAX_TIME_MS
                && dist >= SPACED_STREAM_MIN_DIST
                && dist < SPACED_STREAM_MAX_DIST
            {
                spaced_stream_pattern_count += 1;
            }
        }

        // visual overlap check
        // TODO: consider AR?
        for j in 2..=4 {
            if i >= j {
                let past_obj = &beatmap.hit_objects[i - j];
                let dist = euclidean_distance(current_obj, past_obj);
                if dist < CIRCLE_RADIUS {
                    reading_overlap_count += 1;
                    break;
                }
            }
        }

        // jump consistency (look for squares, triangulars, hexes)
        if i >= 3 {
            angle_consistency_pattern_total += 1;
            let obj_d = current_obj;
            let obj_c = &beatmap.hit_objects[i - 1];
            let obj_b = &beatmap.hit_objects[i - 2];
            let obj_a = &beatmap.hit_objects[i - 3];

            if let (Some(angle1), Some(angle2)) = (
                get_angle(obj_a, obj_b, obj_c),
                get_angle(obj_b, obj_c, obj_d),
            ) {
                let is_similar_angle = (angle1 - angle2).abs() < SIMILAR_ANGLE_THRESHOLD_DEG;
                if is_similar_angle {
                    angle_consistency_count += 1;
                }
            }
        }
    }

    // analyze obj types
    let circle_ratio = beatmap
        .hit_objects
        .iter()
        .filter(|o| o.obj_type == HitObjectType::Circle)
        .count() as f32
        / total_objects;
    let sliders: Vec<&HitObject> = beatmap
        .hit_objects
        .iter()
        .filter(|o| o.obj_type == HitObjectType::Slider)
        .collect();
    let slider_count = sliders.len();

    // look for red points
    // TODO: maybe improve to check slider art
    let sharp_slider_ratio = if slider_count > 0 {
        let sharp_sliders = sliders
            .iter()
            .filter(|s| {
                if let Some(points) = &s.curve_points {
                    for i in 0..points.len().saturating_sub(1) {
                        let is_red_anchor = points[i] == points[i + 1];
                        if is_red_anchor {
                            return true;
                        }
                    }
                }
                false
            })
            .count();
        sharp_sliders as f32 / slider_count as f32
    } else {
        0.0
    };

    let mut features = Vec::with_capacity(FEATURE_COUNT);

    // Feature 1: Stream Pattern Ratio
    features.push(if flow_pattern_total > 0 {
        stream_pattern_count as f32 / flow_pattern_total as f32
    } else {
        0.0
    });

    // Feature 2: Spaced Stream Ratio (alt streams)
    features.push(if flow_pattern_total > 0 {
        spaced_stream_pattern_count as f32 / flow_pattern_total as f32
    } else {
        0.0
    });

    // Feature 2: Reading Overlap Ratio (reading difficulty)
    features.push(if total_objects > 0.0 {
        reading_overlap_count as f32 / total_objects
    } else {
        0.0
    });

    // Feature 3: Angle Consistency Ratio (geometrical patterns)
    features.push(if angle_consistency_pattern_total > 0 {
        angle_consistency_count as f32 / angle_consistency_pattern_total as f32
    } else {
        0.0
    });

    // Feature 4: Grid Adherence Ratio
    features.push(if total_objects > 0.0 {
        grid_snap_count as f32 / total_objects
    } else {
        0.0
    });

    // Feature 5: Circle Ratio (jumps vs sliders)
    features.push(circle_ratio);

    // Feature 6: Sharp Slider Ratio (slider technicality)
    features.push(sharp_slider_ratio);

    // Feature 7: Overall Intensity (normalized average interval)
    const MIN_INTERVAL: f32 = 50.0; // 1/16 notes 300 BPM
    const MAX_INTERVAL: f32 = 500.0; // 1/4 notes 120 BPM
    let avg_interval_ms = if flow_pattern_total > 0 {
        total_time_intervals / flow_pattern_total as f32
    } else {
        0.0
    };
    let normalized_interval = (avg_interval_ms - MIN_INTERVAL) / (MAX_INTERVAL - MIN_INTERVAL);
    features.push(1.0 - normalized_interval.clamp(0.0, 1.0));

    let weights: [f32; FEATURE_COUNT] = [
        1.0, // stream_ratio
        1.0, // spaced_stream_ratio
        1.5, // reading_overlap_ratio, tech
        1.5, // angle_consistency, geometrical
        0.8, // grid_adherence
        0.7, // circle_ratio
        1.2, // sharp_slider_ratio, tech
        1.3, // intensity (difficulty)
    ];

    for i in 0..features.len() {
        features[i] *= weights[i];
    }

    Some(features)
}

fn get_angle(p1: &HitObject, p2: &HitObject, p3: &HitObject) -> Option<f32> {
    let v1_x = p1.x - p2.x;
    let v1_y = p1.y - p2.y;
    let v2_x = p3.x - p2.x;
    let v2_y = p3.y - p2.y;

    let len1_sq = v1_x.powi(2) + v1_y.powi(2);
    let len2_sq = v2_x.powi(2) + v2_y.powi(2);

    if len1_sq > 0.0 && len2_sq > 0.0 {
        let dot_product = v1_x * v2_x + v1_y * v2_y;
        let cross_product = v1_x * v2_y - v1_y * v2_x;
        let angle_rad = cross_product.atan2(dot_product);
        Some(angle_rad.abs().to_degrees())
    } else {
        None
    }
}

fn euclidean_distance(p1: &HitObject, p2: &HitObject) -> f32 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

fn is_grid_snapped(obj: &HitObject, grid_size: f32, tolerance: f32) -> bool {
    let x_rem = obj.x % grid_size;
    let y_rem = obj.y % grid_size;

    (x_rem.abs() < tolerance || (grid_size - x_rem).abs() < tolerance)
        && (y_rem.abs() < tolerance || (grid_size - y_rem).abs() < tolerance)
}
