use crate::{constants::Weight, graph::Node};

pub fn straight_line(src: &Node, dst: &Node) -> Weight {
    // Calculate the distance between two nodes using the Haversine formula
    let lat1 = src.lat.to_radians();
    let lat2 = dst.lat.to_radians();
    let lon1 = src.lon.to_radians();
    let lon2 = dst.lon.to_radians();
    let a = (lat2 - lat1) / 2.0;
    let b = (lon2 - lon1) / 2.0;
    let c = a.sin().powi(2) + lat1.cos() * lat2.cos() * b.sin().powi(2);
    let d = 2.0 * c.sqrt().asin();

    6371.0 * d / 110.0 * 3600.0 // Umrechnung in Sekunden
}
