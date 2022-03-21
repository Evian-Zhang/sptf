/** This method may throw */
async function login(username: string, password: string): Promise<string> {
    return Promise.resolve("");
}

/** This method does not throw */
async function loginWithCookie(): Promise<boolean> {
    return Promise.resolve(true);
}

/** This method may throw */
async function signup(username: string, password: string) {
    return Promise.resolve();
}

export { login, loginWithCookie, signup };
