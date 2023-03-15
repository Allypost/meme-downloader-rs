use std::{
    fmt::{Debug, Display},
};

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::clone_double_ref)]
pub fn check_results<TVal: Debug, TErr: Display>(
    result: Vec<Result<TVal, TErr>>,
) -> Result<Vec<TVal>, String> {
    if result.iter().any(Result::is_err) {
        let mapped = result.iter().filter(|x| x.is_err()).map(|x| {
            return x.as_ref().unwrap_err().clone();
        });

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