import React, { useState } from 'react';
import {
  Form,
  Input,
  Button,
  Modal,
} from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { login, loginWithCookie } from './custom-utils/conn';

const { Item: FormItem } = Form;

export interface LoginProps {
  setAuthTokenAndToFileBrowser: (authToken: string) => void,
  toSignup: () => void,
}

function Login(props: LoginProps) {
  const [validating, setValidating] = useState(false);
  const [loginForm] = Form.useForm();
  window.sptfAPI.getCookie()
    .then((authToken) => {
      if (authToken) {
        setValidating(true);
        loginWithCookie()
          .then((isSuccess) => {
            setValidating(false);
            if (isSuccess) {
              props.setAuthTokenAndToFileBrowser(authToken);
            }
          })
      }
    })
  
  function onFinish() {
    setValidating(true);
    const username = loginForm.getFieldValue("username");
    const password = loginForm.getFieldValue("password");
    login(username, password)
      .then((authToken) => {
        setValidating(false);
        props.setAuthTokenAndToFileBrowser(authToken);
      });
  }

  function onGoToSignupPageButtonPressed() {
    props.toSignup();
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
        visible={validating}
        centered
        footer={null}
        closable={false}
        maskClosable={false}
      >
        正在验证身份...
      </Modal>
    </React.Fragment>
  );
};

export default Login;
