import React, { useState, useEffect } from 'react';
import {
  Form,
  Input,
  Button,
  Modal,
  message
} from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';

const { Item: FormItem } = Form;

export interface LoginProps {
  setAuthTokenAndToFileBrowser: (authToken: string) => void,
  toSignup: () => void,
  loginShouldUseCookie: boolean,
}

enum LoginValidationStatus {
  NoLogin,
  Validating,
  Invalid,
}

function Login(props: LoginProps) {
  const [validating, setValidating] = useState(LoginValidationStatus.NoLogin);
  const [loginForm] = Form.useForm();
  useEffect(() => {
    if (props.loginShouldUseCookie) {
      window.sptfAPI.getCookie()
        .then((authToken) => {
          if (authToken) {
            document.cookie = `SPTF_AUTH=${authToken};samesite=none;expires=${new Date(2200, 1).toUTCString}`;
            setValidating(LoginValidationStatus.Validating);
            window.sptfAPI.loginWithCookie()
              .then((isSuccess) => {
                if (isSuccess) {
                  setValidating(LoginValidationStatus.NoLogin);
                  props.setAuthTokenAndToFileBrowser(authToken);
                } else {
                  setValidating(LoginValidationStatus.Invalid);
                }
              })
              .catch(() => {
                setValidating(LoginValidationStatus.NoLogin);
                window.sptfAPI.removeCookie();
              })
          }
        })
    }
  }, [props.loginShouldUseCookie]);
  
  function onFinish() {
    setValidating(LoginValidationStatus.Validating);
    const username = loginForm.getFieldValue("username");
    const password = loginForm.getFieldValue("password");
    window.sptfAPI.login(username, password)
      .then((authToken) => {
        setValidating(LoginValidationStatus.NoLogin);
        props.setAuthTokenAndToFileBrowser(authToken);
      })
      .catch((reason) => {
        setValidating(LoginValidationStatus.Invalid);
        message.error(reason);
      });
  }

  function onGoToSignupPageButtonPressed() {
    props.toSignup();
  }

  function onLoginFailedButtonPressed() {
    setValidating(LoginValidationStatus.NoLogin);
    loginForm.resetFields();
  }

  return (
    <React.Fragment>
      <Form layout='horizontal' style={{ width: "300px" }} form={loginForm} onFinish={onFinish}>
        <FormItem
          name="username"
          rules={[{ required: true, message: '?????????????????????' }]}
        >
          <Input prefix={<UserOutlined className="site-form-item-icon" />} placeholder="??????????????????" />
        </FormItem>

        <Form.Item
          name="password"
          rules={[{ required: true, message: '??????????????????' }]}
        >
          <Input
            prefix={<LockOutlined className="site-form-item-icon" />}
            type="password"
            placeholder="???????????????"
          />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" style={{width: "100%"}}>
            ??????
          </Button>
          ??????<Button type="link" onClick={onGoToSignupPageButtonPressed}>???????????????</Button>
        </Form.Item>
      </Form>

      <Modal
        visible={validating === LoginValidationStatus.Validating}
        centered
        footer={null}
        closable={false}
        maskClosable={false}
      >
        ??????????????????...
      </Modal>
      <Modal
        visible={validating === LoginValidationStatus.Invalid}
        onOk={onLoginFailedButtonPressed}
        centered
        okText={"??????"}
        cancelButtonProps={{ style: { display: "none" } }}
      >
        ???????????????
      </Modal>
    </React.Fragment>
  );
};

export default Login;
