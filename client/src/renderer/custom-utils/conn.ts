async function login(username: string, password: string): Promise<string> {
    return Promise.resolve("");
}

async function loginWithCookie(): Promise<boolean> {
    return Promise.resolve(true);
}

async function signup(username: string, password: string) {
    return Promise.resolve();
}

export { login, loginWithCookie, signup };
