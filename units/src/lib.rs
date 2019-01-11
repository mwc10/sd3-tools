use std::fmt;
use std::str::FromStr;
use serde::de::{self, Visitor, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum SIError {
    #[fail(display = "Cannot convert from {:?} to {:?}", _0, _1)]
    IncompatibleTypes(UnitType, UnitType),
    #[fail(display = "Unknown SI unit <{}>", _0)]
    UnkType(String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SIUnit {
    pg_ml,
    ng_ml,
    mg_ml,
    mg_dl,
    g_l,

    ml,
    ul,
    dl,
    l,

    ng,
    g,

    g_day,
    ng_day,

    g_day_cell,
    ng_day_cell,
    ng_day_millioncells,
}

impl SIUnit {
    fn unit_type(&self) -> UnitType {
        use self::SIUnit::*;
        use self::UnitType::*;
        match self {
            pg_ml | ng_ml | mg_ml | mg_dl | g_l
                => Concentration,
            ul | ml | dl | l 
                => Volume,
            g | ng
                => Mass,
            ng_day | g_day
                => Rate,
            ng_day_cell | ng_day_millioncells | g_day_cell
                => CellNormalized,
        }
    }
    /*
    fn si_base(&self) -> Self {
        use self::SIUnit::*;
        use self::UnitType::*;

        match self.unit_type() {
            Concentration => g_l,
            Volume => l,
            Rate => g_day,
            CellNormalized => g_day_cell,
            Mass => g,
        }
    }*/

    fn as_str(&self) -> &'static str {
        use self::SIUnit::*;

        match self {
            pg_ml => "pg/mL",
            ng_ml => "ng/mL",
            mg_ml => "mg/mL",
            mg_dl => "mg/dL",
            g_l => "g/L",

            ml => "mL",
            ul => "µL",
            dl => "dL",
            l => "L",

            g => "g",
            ng => "ng",

            g_day => "g/day",
            ng_day => "ng/day",

            g_day_cell => "g/day/cell",
            ng_day_cell => "ng/day/cell",
            ng_day_millioncells => "ng/day/10^6 cells",
        }
    }
    /// Factor to put this unit into base SI unit
    fn si_factor(&self) -> f64 {
        use self::SIUnit::*;

        match self {
            pg_ml => 1e-9,
            ng_ml => 1e-6,
            mg_ml => 1.0,
            mg_dl => 1e-2,
            g_l => 1.0,

            ml => 1e-3,
            ul => 1e-6,
            dl => 1e-1,
            l => 1.0,

            g => 1.0,
            ng => 1e-9,

            g_day => 1.0,
            ng_day => 1e-9,
            /* These seem off... */
            g_day_cell => 1.0,
            ng_day_cell => 1e-9,
            ng_day_millioncells => 1.0 / (1_000_000_000.0 * 1_000_000_000.0),
        }
    }
}

impl fmt::Display for SIUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SIUnit {
    type Err = SIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::SIUnit::*;

        match s {
            "pg/mL" | "pg/ml" => Ok(pg_ml),
            "ng/mL" | "ng/ml" => Ok(ng_ml),
            "mg/mL" | "mg/ml" => Ok(mg_ml),
            "mg/dL" | "mg/dl" => Ok(mg_dl),
            "g/L" | "g/l" => Ok(g_l),

            "mL" | "ml" => Ok(ml),
            "µL" | "µl" | "ul" | "uL" => Ok(ul),
            "dL" | "dl" => Ok(dl),
            "L" | "l" => Ok(l),

            "g" => Ok(g),
            "ng" => Ok(ng),

            "g/day" => Ok(g_day),
            "ng/day" => Ok(ng_day),

            "g/day/cell" => Ok(g_day_cell),
            "ng/day/cell" => Ok(ng_day_cell),
            "ng/day/10^6 cells" | "ng/day/10^6cells" => Ok(ng_day_millioncells),
            
            _ => Err(SIError::UnkType(s.to_string())),
        }
    }
}

impl Serialize for SIUnit {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SIUnit {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        d.deserialize_str(SIUnitVisitor)
    }
}
struct SIUnitVisitor;

impl<'de> Visitor<'de> for SIUnitVisitor {
    type Value = SIUnit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A typical SI concentration unit")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where E: de::Error
    {
        Self::Value::from_str(s)
            .map_err( |e| E::custom(format!("{}",e)) )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnitType {
    Concentration,
    Volume,
    CellNormalized,
    Rate,
    Mass,
}

pub fn convert((val, unit): (f64, SIUnit), to: SIUnit) -> Result<f64, SIError> {
    let from_type = unit.unit_type();
    let to_type = to.unit_type();
    if from_type != to_type {
        return Err(SIError::IncompatibleTypes(from_type, to_type));
    }
    let from_fact = unit.si_factor();
    let to_fact = to.si_factor().recip();
    
    Ok(val * (from_fact * to_fact))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::double_comparable;
    const TOL: f64 = 1e-9;

    #[test]
    fn mass_conversion() {
        use self::SIUnit::*;

        assert!(double_comparable(convert((1e9, ng), g).unwrap(), 1.0, TOL), "10^9 ng to g");
        assert!(double_comparable(convert((1e-9, g), ng).unwrap(), 1.0, TOL), "10^-9 g to ng");
        assert!(double_comparable(convert((100.0, ng), ng).unwrap(), 100.0, TOL), "100 ng to ng");
        assert!(double_comparable(convert((25.0, g), g).unwrap(), 25.0, TOL), "25 g to g");
    }

    #[test]
    fn volume_conversion() {
        use self::SIUnit::*;

        assert!(double_comparable(convert((100.0, ul), ml).unwrap(), 0.1, TOL), "100 ul to ml");
        assert!(double_comparable(convert((50.0, dl), ul).unwrap(), 5.0e6, TOL), "50 dl to ul");
        assert!(double_comparable(convert((10.0, dl), l).unwrap(), 1.0, TOL), "10 dl to l");
        assert!(double_comparable(convert((385.0, ml), dl).unwrap(), 3.85, TOL), "385 ml to dl");
        assert!(double_comparable(convert((2054.0, ml), l).unwrap(), 2.054, TOL), "2054 ml to l");
    }

    #[test]
    fn concentration_conversion() {
        use self::SIUnit::*;

        assert!(double_comparable(convert((100.0, pg_ml), g_l).unwrap(), 100e-9, TOL), "100 pg_ml to g_l");
        assert!(double_comparable(convert((20.0, ng_ml), g_l).unwrap(), 20e-6, TOL), "20 ug_ml to g_l");
        assert!(double_comparable(convert((32.0, mg_ml), g_l).unwrap(), 32.0, TOL), "32 mg_ml to g_l");
        assert!(double_comparable(convert((1.0, mg_dl), g_l).unwrap(), 1e-2, TOL), "1 mg_dl to g_l");
    }
}

