use std::fmt::{Debug, Display};

#[allow(suspicious_double_ref_op)]
#[allow(dead_code)]
pub fn check_results<TVal: Debug, TErr: Display>(
    result: Vec<Result<TVal, TErr>>,
) -> Result<Vec<TVal>, String> {
    if result.iter().any(Result::is_err) {
        let mapped = result
            .iter()
            .filter(|x| x.is_err())
            .map(|x| x.as_ref().unwrap_err());

        let mut ret = vec![];
        for r in mapped {
            ret.push(r.to_string());
        }

        return Err(ret.join(", "));
    }

    let mut ret = vec![];
    for r in result.into_iter().flatten() {
        ret.push(r);
    }

    Ok(ret)
}

pub fn option_contains<T: Eq>(option: &Option<T>, contains: &T) -> bool {
    option.as_ref().map_or(false, |value| contains == value)
}
