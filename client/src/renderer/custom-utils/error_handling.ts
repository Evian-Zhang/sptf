function handleErrorCode(error_code: number): string {
    switch (error_code) {
        case 0x0: return "服务器内部错误";
        case 0x1: return "用户不存在";
        case 0x2: return "密码不正确";
        case 0x3: return "Cookie已过期";
        case 0x4: return "Cookie设置失败";
        case 0x5: return "Cookie验证失败";
        case 0x6: return "您的权限不够";
        case 0x7: return "文件传输格式错误";
        case 0x8: return "用户名已存在";
        default: return "未知错误";
    }
}

interface SPTFError {
    errorCode: number
}

/** After we checking the response does not 200, call this method, which
 * will **throw** the description
 */
async function handleNonOkHttpResponse(response: Response) {
    if (response.status != 500) {
        throw "未知错误";
    }
    let response_json: SPTFError;
    try {
        response_json = await response.json();
    } catch {
        throw "未知错误";
    }
    if (!response_json || !response_json.errorCode) {
        throw "未知错误";
    }
    throw handleErrorCode(response_json.errorCode);
}

export { handleErrorCode, handleNonOkHttpResponse };
