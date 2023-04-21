use std::str::FromStr;

// Only this road types are inclued in the graph
#[derive(Debug, Clone, Copy)]
pub enum RoadType {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    MotorwayLink,
    TrunkLink,
    PrimaryLink,
    SecondaryLink,
    Road,
    Unclassified,
    Residential,
    Unsurfaced,
    LivingStreet,
    Service,
}

impl RoadType {
    // Returns the average road velocity in km/h
    // From https://ad-wiki.informatik.uni-freiburg.de/teaching/EfficientRoutePlanningSS2011/RoadTypesAndSpeeds
    pub fn velocity(&self) -> f64 {
        match self {
            RoadType::Motorway => 110.0,
            RoadType::Trunk => 110.0,
            RoadType::Primary => 70.0,
            RoadType::Secondary => 60.0,
            RoadType::Tertiary => 50.0,
            RoadType::MotorwayLink => 50.0,
            RoadType::TrunkLink => 50.0,
            RoadType::PrimaryLink => 50.0,
            RoadType::SecondaryLink => 50.0,
            RoadType::Road => 40.0,
            RoadType::Unclassified => 40.0,
            RoadType::Residential => 30.0,
            RoadType::Unsurfaced => 30.0,
            RoadType::LivingStreet => 10.0,
            RoadType::Service => 5.0,
        }
    }
}

impl FromStr for RoadType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "motorway" => Ok(RoadType::Motorway),
            "trunk" => Ok(RoadType::Trunk),
            "primary" => Ok(RoadType::Primary),
            "secondary" => Ok(RoadType::Secondary),
            "tertiary" => Ok(RoadType::Tertiary),
            "motorway_link" => Ok(RoadType::MotorwayLink),
            "trunk_link" => Ok(RoadType::TrunkLink),
            "primary_link" => Ok(RoadType::PrimaryLink),
            "secondary_link" => Ok(RoadType::SecondaryLink),
            "road" => Ok(RoadType::Road),
            "unclassified" => Ok(RoadType::Unclassified),
            "residential" => Ok(RoadType::Residential),
            "unsurfaced" => Ok(RoadType::Unsurfaced),
            "living_street" => Ok(RoadType::LivingStreet),
            "service" => Ok(RoadType::Service),
            _ => Err(format!("Failed to parse road type '{}'", s)),
        }
    }
}
