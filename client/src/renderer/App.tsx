import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
import React, { useState } from 'react'
import { Layout } from 'antd';
import './App.css';
import Login from './login'
import Signup from './signup';

const { Header, Content } = Layout;

enum HomepageComponentStatus {
  Login,
  Signup,
  Filebrowser,
}

const Index = () => {
  const [authToken, setAuthToken] = useState<string | null>(null);
  const [homepageComponentStatus, setHomepageComponentStatus] = useState(HomepageComponentStatus.Login);
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

  function subComponent(homepageComponentStatus: HomepageComponentStatus) {
    switch (homepageComponentStatus) {
      case HomepageComponentStatus.Login: {
        return (
          <Login
            setAuthTokenAndToFileBrowser={childComponentSetAuthTokenAndToFileBrowser}
            toSignup={toSignup}
          />
        );
      }
      case HomepageComponentStatus.Signup: {
        return (
          <Signup
            toLogin={toLogin}
          />
        )
      }
      case HomepageComponentStatus.Filebrowser: break;
    }
  }

  return (
    <Layout style={{minHeight: "100vh"}}>
      <Header>
        <h1 style={{color: "white"}}>SPTF</h1>
      </Header>
      <Content style={{ padding: 48, 
          width: "100%",
          height: "100%",
          position: "relative",
          display: "flex",
          justifyContent: "center", alignItems: "center" }}>
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
