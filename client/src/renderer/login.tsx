import React, { useState, useEffect } from 'react';
import {
  Form,
  Input,
  Button,
  Modal,
  message
} from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { login, loginWithCookie } from './custom-utils/conn';

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
            loginWithCookie()
              .then((isSuccess) => {
                if (isSuccess) {
                  setValidating(LoginValidationStatus.NoLogin);
                  props.setAuthTokenAndToFileBrowser(authToken);
                } else {
                  setValidating(LoginValidationStatus.Invalid);
                }
              })
          }
        })
    }
  }, [props.loginShouldUseCookie]);
  
  function onFinish() {
    setValidating(LoginValidationStatus.Validating);
    const username = loginForm.getFieldValue("username");
    const password = loginForm.getFieldValue("password");
    login(username, password)
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
          rules={[{ required: true, message: '用户名不得为空' }]}
        >
          <Input prefix={<UserOutlined className="site-form-item-icon" />} placeholder="请输入用户名" />
        </FormItem>

        <Form.Item
          name="password"
          rules={[{ required: true, message: '密码不得为空' }]}
        >
          <Input
            prefix={<LockOutlined className="site-form-item-icon" />}
            type="password"
            placeholder="请输入密码"
          />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" style={{width: "100%"}}>
            登录
          </Button>
          或者<Button type="link" onClick={onGoToSignupPageButtonPressed}>现在注册！</Button>
        </Form.Item>
      </Form>

      <Modal
        visible={validating === LoginValidationStatus.Validating}
        centered
        footer={null}
        closable={false}
        maskClosable={false}
      >
        正在验证身份...
      </Modal>
      <Modal
        visible={validating === LoginValidationStatus.Invalid}
        onOk={onLoginFailedButtonPressed}
        centered
        okText={"确认"}
        cancelButtonProps={{ style: { display: "none" } }}
      >
        验证失败！
      </Modal>
    </React.Fragment>
  );
};

export default Login;
