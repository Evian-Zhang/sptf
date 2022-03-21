async function login(username: string, password: string): Promise<string> {
    return Promise.resolve("");
}

async function loginWithCookie(): Promise<boolean> {
    return Promise.resolve(true);
}

export { login, loginWithCookie };
