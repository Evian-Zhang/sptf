const {session} = require('electron')

const COOKIE_NAME = "SPTF_AUTH";
const DESTINATION_URL = "http://evian-workstation.local";

async function getCookie(): Promise<string | null> {
    try {
        const authTokens = await session.defaultSession.cookies.get({"name": COOKIE_NAME});
        if (authTokens.length === 0) {
            return Promise.resolve(null);
        } else {
            return Promise.resolve(authTokens[0].value)
        }
    } catch {
        return Promise.resolve(null)
    }
}

async function setCookie(authToken: string) {
    const EXPIRATION_DATE = new Date(2200, 1).getTime() / 1000;
    const cookie = {
        name: COOKIE_NAME,
        value: authToken,
        url: DESTINATION_URL,
        session: false,
        expirationDate: EXPIRATION_DATE
    };
    await session.defaultSession.cookies.set(cookie);
}

async function removeCookie() {
    await session.defaultSession.cookies.remove(DESTINATION_URL, COOKIE_NAME);
}

export { getCookie, setCookie, removeCookie };
