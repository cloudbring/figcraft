use std::collections::BTreeSet;

use figcraft_geom::kurbo::{BezPath, CubicBez, ParamCurve, PathEl, Point};

type Key = (u64, u64);

fn key(point: Point) -> Key {
    ((point.x + 0.0).to_bits(), (point.y + 0.0).to_bits())
}

pub(crate) fn anchors<'a>(paths: impl IntoIterator<Item = &'a BezPath>) -> BTreeSet<Key> {
    paths.into_iter().flat_map(|path| path.elements().iter().filter_map(PathEl::end_point)).map(key).collect()
}

fn merge(first: CubicBez, second: CubicBez) -> Option<CubicBez> {
    let before = (first.p3 - first.p2).hypot();
    let after = (second.p1 - second.p0).hypot();
    let t = before / (before + after);
    if !(1e-6..1.0 - 1e-6).contains(&t) {
        return None;
    }
    let combined = CubicBez::new(first.p0, first.p0 + (first.p1 - first.p0) / t, second.p3 + (second.p2 - second.p3) / (1.0 - t), second.p3);
    let left = combined.subsegment(0.0..t);
    let right = combined.subsegment(t..1.0);
    let scale = 1.0 + first.p0.to_vec2().hypot().max(second.p3.to_vec2().hypot());
    let tolerance = scale * 1e-10;
    let matches = |a: CubicBez, b: CubicBez| {
        a.p0.distance(b.p0) <= tolerance && a.p1.distance(b.p1) <= tolerance && a.p2.distance(b.p2) <= tolerance && a.p3.distance(b.p3) <= tolerance
    };
    (matches(left, first) && matches(right, second)).then_some(combined)
}

pub(crate) fn refit(path: &BezPath, anchors: &BTreeSet<Key>) -> BezPath {
    let mut result = BezPath::new();
    let mut current = Point::ORIGIN;
    let mut start = Point::ORIGIN;
    let mut pending: Option<CubicBez> = None;
    for element in path.elements() {
        if let PathEl::CurveTo(control1, control2, end) = *element {
            let next = CubicBez::new(current, control1, control2, end);
            if let Some(previous) = pending.take() {
                if !anchors.contains(&key(current))
                    && let Some(combined) = merge(previous, next)
                {
                    pending = Some(combined);
                } else {
                    result.curve_to(previous.p1, previous.p2, previous.p3);
                    pending = Some(next);
                }
            } else {
                pending = Some(next);
            }
            current = end;
            continue;
        }
        if let Some(curve) = pending.take() {
            result.curve_to(curve.p1, curve.p2, curve.p3);
        }
        result.push(*element);
        match *element {
            PathEl::MoveTo(point) => {
                start = point;
                current = point;
            }
            PathEl::LineTo(point) | PathEl::QuadTo(_, point) => current = point,
            PathEl::ClosePath => current = start,
            PathEl::CurveTo(..) => {}
        }
    }
    if let Some(curve) = pending {
        result.curve_to(curve.p1, curve.p2, curve.p3);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refits_exact_splits_but_keeps_original_anchors() {
        let curve = CubicBez::new((0.0, 0.0), (2.0, 4.0), (4.0, 4.0), (6.0, 0.0));
        let (left, right) = curve.subdivide();
        let mut path = BezPath::new();
        path.move_to(curve.p0);
        path.curve_to(left.p1, left.p2, left.p3);
        path.curve_to(right.p1, right.p2, right.p3);
        assert_eq!(refit(&path, &BTreeSet::new()).segments().count(), 1);
        assert_eq!(refit(&path, &anchors([&path])).segments().count(), 2);
    }
}
