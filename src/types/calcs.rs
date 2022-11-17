use libm;

#[derive(PartialEq, Clone, Copy)]
pub enum QualityLevel {
    Poor,
    Ok,
    Good,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Suggestion {
    Remove(f64),
    Add(f64),
    None,
}

impl QualityLevel {
    pub fn code(&self) -> &str {
        match *self {
            QualityLevel::Good => "OK",
            QualityLevel::Ok => "ME",
            QualityLevel::Poor => "XD",
        }
    }
}

// HARDNESS:
// 0-60: BAD
// 60-80: OKAY
// 80-100: GOOD
// 100-150: OKAY
// 150+: BAD

pub fn eval_hardness(hardness: f64) -> QualityLevel {
    if hardness > 150.0 || hardness < 60.0 {
        QualityLevel::Poor
    } else if hardness > 100.0 || hardness < 80.0 {
        QualityLevel::Ok
    } else {
        QualityLevel::Good
    }
}

pub fn improve_hardness(hardness: f64) -> Suggestion {
    match hardness {
        h if h > 100.0 => Suggestion::Remove(hardness - 100.0),
        h if h < 80.0 => Suggestion::Add(80.0 - hardness),
        _ => Suggestion::None,
    }
}

// pH:
// 0-6.5: BAD
// 6.5-6: OKAY
// 6-8: GOOD
// 8-8.5: OKAY
// 8.5+: BAD

pub fn eval_ph(ph: f64) -> QualityLevel {
    if ph > 8.5 || ph < 6.0 {
        QualityLevel::Poor
    } else if ph > 8.0 || ph < 6.5 {
        QualityLevel::Ok
    } else {
        QualityLevel::Good
    }
}

pub fn improve_ph(ph: f64) -> Suggestion {
    // returns whether to add or remove BASE
    let h_conc = libm::pow(10.0, -ph);
    let ph_6_conc = 1e-6;
    let ph_8_conc = 1e-8;

    match h_conc {
        h if h > ph_8_conc => Suggestion::Add(h - ph_8_conc),
        h if h < ph_6_conc => Suggestion::Remove(ph_6_conc - h),
        _ => Suggestion::None,
    }
}

// COND:
// 0-10: BAD
// 10-50: OKAY
// 50-120: GOOD
// 120-180: OKAY
// 180+: BAD

pub fn eval_cond(cond: f64) -> QualityLevel {
    // in units of mg/L
    let salinity = (0.7317 * cond - 3.7635) * 0.55;
    if salinity > 180.0 || salinity < 10.0 {
        QualityLevel::Poor
    } else if salinity > 120.0 || salinity < 50.0 {
        QualityLevel::Ok
    } else {
        QualityLevel::Good
    }
}

pub fn improve_cond(cond: f64) -> Suggestion {
    // returns whether to add or remove total
    // dissolved salts (TDS)

    // in units of mg/L
    let salinity = (0.7317 * cond - 3.7635) * 0.55;

    match salinity {
        s if s > 120.0 => Suggestion::Remove(s - 120.0),
        s if s < 50.0 => Suggestion::Add(50.0 - s),
        _ => Suggestion::None,
    }
}
