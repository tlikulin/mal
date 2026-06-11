use crate::types::{MalError, MalResult, MalType};

pub fn add(args: Vec<MalType>) -> MalResult {
    let Some(nums) = args
        .into_iter()
        .map(MalType::into_number)
        .collect::<Option<Vec<i32>>>()
    else {
        return Err(MalError::EvalError(
            "non-number arguments to '+'".to_string(),
        ));
    };

    Ok(MalType::Number(nums.iter().sum()))
}

pub fn sub(args: Vec<MalType>) -> MalResult {
    let Some(nums) = args
        .into_iter()
        .map(MalType::into_number)
        .collect::<Option<Vec<i32>>>()
    else {
        return Err(MalError::EvalError(
            "non-number arguments to '-'".to_string(),
        ));
    };

    if nums.len() == 2 {
        Ok(MalType::Number(nums[0] - nums[1]))
    } else {
        Err(MalError::EvalError("'-' expects 2 args".to_string()))
    }
}

pub fn mult(args: Vec<MalType>) -> MalResult {
    let Some(nums) = args
        .into_iter()
        .map(MalType::into_number)
        .collect::<Option<Vec<i32>>>()
    else {
        return Err(MalError::EvalError(
            "non-number arguments to '*'".to_string(),
        ));
    };

    Ok(MalType::Number(nums.iter().product()))
}

pub fn div(args: Vec<MalType>) -> MalResult {
    let Some(nums) = args
        .into_iter()
        .map(MalType::into_number)
        .collect::<Option<Vec<i32>>>()
    else {
        return Err(MalError::EvalError(
            "non-number arguments to '/'".to_string(),
        ));
    };

    if nums.len() == 2 {
        Ok(MalType::Number(nums[0] / nums[1]))
    } else {
        Err(MalError::EvalError("'/' expects 2 args".to_string()))
    }
}
