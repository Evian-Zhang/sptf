import { sptf } from '../../renderer/custom-utils/protos';
import { handleErrorCode } from '../../renderer/custom-utils/error_handling';

const { net, session } = require('electron');

const SERVER_DOMAIN = "https://evian-workstation.local:8766";

interface SPTFError {
    errorCode: number
}

/** After we checking the response does not 200, call this method, which
 * will **return** the description
 */
function handleNonOkHttpResponse(dataBuffer: Buffer, statusCode: number) {
    if (statusCode != 500) {
        return "未知错误";
    }
    const data = dataBuffer.toString();
    let response_json: SPTFError;
    try {
        response_json = JSON.parse(data);
    } catch {
        return "未知错误";
    }
    if (!response_json || !response_json.errorCode) {
        return "未知错误";
    }
    return handleErrorCode(response_json.errorCode);
}

/** This method may throw */
async function login(username: string, password: string): Promise<string> {
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/login`,
            session: session.defaultSession
        });
        req.setHeader("Content-Type", "application/json");
        req.write(JSON.stringify({username: username, password: password}));
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                response.on("data", (data) => {
                    let response;
                    try {
                        response = JSON.parse(data.toString());
                        resolve(response.authToken);
                    } catch {
                        reject("网络错误");
                    }
                })
            }
            response.on("error", () => {
                reject("网络错误");
            });
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

async function logout(): Promise<void> {
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/logout`,
            session: session.defaultSession,
            useSessionCookies: true
        });
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                resolve();
            }
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

/** This method does not throw */
async function loginWithCookie(): Promise<boolean> {
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/login_with_cookie`,
            session: session.defaultSession,
            useSessionCookies: true
        });
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                resolve(true);
            }
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

/** This method may throw */
async function signup(username: string, password: string): Promise<void> {
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/signup`,
            session: session.defaultSession
        });
        req.setHeader("Content-Type", "application/json");
        req.write(JSON.stringify({username: username, password: password}));
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                resolve()
            }
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

async function uploadFiles(currentDir: string, files: {fileName: string, content: Blob}[]): Promise<void> {
    let uploadedFiles = [];
    for (let file of files) {
        const dataArray = await new Response(file.content).arrayBuffer();
        const content = new Uint8Array(dataArray)
        uploadedFiles.push({fileName: file.fileName, content: content});
    }
    const fileUploadRequest = sptf.FileUploadRequest.encode({
        dirPath: currentDir,
        uploadedFile: uploadedFiles
    });
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/upload`,
            session: session.defaultSession,
            useSessionCookies: true,
        });
        req.write(Buffer.from(fileUploadRequest.finish()));
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                resolve()
            }
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

async function makeDirectory(directoryPath: string): Promise<void> {
    return new Promise(((resolve, reject) => {
        const req = net.request({
            method: "POST",
            url: `${SERVER_DOMAIN}/make_directory`,
            session: session.defaultSession,
            useSessionCookies: true
        });
        req.write(JSON.stringify({directoryPath: directoryPath}));
        req.on("response", (response) => {
            if (response.statusCode !== 200) {
                response.on("data", (data) => {
                    const returnError = handleNonOkHttpResponse(data, response.statusCode);
                    reject(returnError);
                })
            } else {
                resolve();
            }
        });
        req.on("error", () => {
          reject("网络错误");
        })
        req.end();
    }));
}

export { login, loginWithCookie, logout, signup, uploadFiles, makeDirectory };
