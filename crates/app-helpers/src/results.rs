use std::{
    fmt::{Debug, Display},
    result::Result,
    string::ToString,
};

#[allow(suspicious_double_ref_op)]
#[allow(dead_code)]
pub fn check_results<TVal: Debug, TErr: Display>(
    result: Vec<Result<TVal, TErr>>,
) -> Result<Vec<TVal>, String> {
    let (success, err): (Vec<_>, Vec<_>) = result.into_iter().partition(|x| x.as_ref().is_ok());

    if !err.is_empty() {
        let ret = err
            .into_iter()
            .filter_map(Result::err)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        return Err(ret);
    }

    Ok(success.into_iter().flatten().collect())
}

pub fn option_contains<T: Eq>(option: &Option<T>, contains: &T) -> bool {
    option.as_ref().map_or(false, |value| contains == value)
}
