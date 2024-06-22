/// Returns the distance between two points on the Earth's surface using the Haversine formula.
/// This is the shortest distance over the Earth's surface.
/// It follows a great circle.
pub fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    const R: f32 = 6371.0; // Earth's radius in kilometers

    let dlat1 = lat1.to_radians();
    let dlat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2) + dlat1.cos() * dlat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    R * c * 1000.0 // Convert to meters
}
