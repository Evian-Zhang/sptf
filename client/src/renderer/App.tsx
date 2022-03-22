import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
import React, { useState } from 'react'
import { Layout } from 'antd';
import './App.css';
import Login from './login';
import Signup from './signup';
import FileBrowser from './fileBrowser';

const { Header, Content } = Layout;

enum HomepageComponentStatus {
  Login,
  Signup,
  Filebrowser,
}

const Index = () => {
  const [authToken, setAuthToken] = useState<string | null>(null);
  const [homepageComponentStatus, setHomepageComponentStatus] = useState(HomepageComponentStatus.Login);
  const [loginShouldUseCookie, setLoginShouldUseCookie] = useState(true);
  const toSignup = () => {
    setHomepageComponentStatus(HomepageComponentStatus.Signup);
  };
  const toLogin = () => {
    setHomepageComponentStatus(HomepageComponentStatus.Login);
  };
  const childComponentSetAuthTokenAndToFileBrowser = (authToken: string) => {
    setAuthToken(authToken);
    setHomepageComponentStatus(HomepageComponentStatus.Filebrowser);
  }
  const onFileBrowserAuthFailed = () => {
    setLoginShouldUseCookie(false);
    window.sptfAPI.removeCookie();
    setAuthToken(null);
    setHomepageComponentStatus(HomepageComponentStatus.Login);
  };
  const onFileBrowserConnectFailed = () => {
    setLoginShouldUseCookie(false);
    setHomepageComponentStatus(HomepageComponentStatus.Login);
  };

  function subComponent(homepageComponentStatus: HomepageComponentStatus) {
    switch (homepageComponentStatus) {
      case HomepageComponentStatus.Login: {
        return (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              alignItems: "center"
            }}
          >
            <Login
              setAuthTokenAndToFileBrowser={childComponentSetAuthTokenAndToFileBrowser}
              toSignup={toSignup}
              loginShouldUseCookie={loginShouldUseCookie}
            />
          </div>
        );
      }
      case HomepageComponentStatus.Signup: {
        return (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              alignItems: "center"
            }}
          >
            <Signup
              toLogin={toLogin}
            />
          </div>
        );
      }
      case HomepageComponentStatus.Filebrowser: {
        return (
          <FileBrowser
            authToken={authToken ?? ""}
            onAuthFailed={onFileBrowserAuthFailed}
            onWebsocketFailed={onFileBrowserConnectFailed}
          />
        );
      }
    }
  }

  return (
    <Layout style={{minHeight: "100vh"}}>
      <Header>
        <h1 style={{color: "white"}}>SPTF</h1>
      </Header>
      <Content
        style={{
          padding: 48, 
          width: "100%",
          height: "100%",
          position: "relative",
        }}
      >
        {subComponent(homepageComponentStatus)}
      </Content>
    </Layout>
  );
};

export default function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Index />} />
      </Routes>
    </Router>
  );
}
