import React, { useState } from 'react';
import {
  Form,
  Input,
  Button,
  Modal,
  message,
} from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { signup } from './custom-utils/conn';

const { Item: FormItem } = Form;

export interface SignupProps {
  toLogin: () => void,
}

enum SignupValidationStatus {
  NoSignup,
  Validating,
  Valid,
  Invalid,
}

function Signup(props: SignupProps) {
  const [validationStatus, setValidationStatus] = useState(SignupValidationStatus.NoSignup);
  const [signupForm] = Form.useForm();
  
  function onFinish() {
    setValidationStatus(SignupValidationStatus.Validating);
    const username = signupForm.getFieldValue("username");
    const password = signupForm.getFieldValue("password");
    signup(username, password)
      .then(() => {
        setValidationStatus(SignupValidationStatus.Valid);
      })
      .catch((reason) => {
        setValidationStatus(SignupValidationStatus.Invalid);
        message.error(reason);
      });
  }

  function onGoToLoginPageButtonPressed() {
    props.toLogin();
  }

  function onSignupSuccessButtonPressed() {
    setValidationStatus(SignupValidationStatus.NoSignup);
    props.toLogin();
  }

  function onSignupFailedButtonPressed() {
    setValidationStatus(SignupValidationStatus.NoSignup);
    signupForm.resetFields();
  }

  return (
    <React.Fragment>
      <Form layout='horizontal' style={{ width: "300px" }} form={signupForm} onFinish={onFinish}>
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
        <Form.Item
          name="confirm"
          dependencies={['password']}
          hasFeedback
          rules={[
            {
              required: true,
              message: '请再次输入密码！',
            },
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!value || getFieldValue('password') === value) {
                  return Promise.resolve();
                }
                return Promise.reject(new Error('密码不一致'));
              },
            }),
          ]}
        >
          <Input
              prefix={<LockOutlined className="site-form-item-icon" />}
              type="password"
              placeholder="请再次输入密码"
          />
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" style={{width: "100%"}}>
            注册
          </Button>
          或者<Button type="link" onClick={onGoToLoginPageButtonPressed}>返回登录界面</Button>
        </Form.Item>
      </Form>

      <Modal
        visible={validationStatus === SignupValidationStatus.Validating}
        centered
        footer={null}
        closable={false}
        maskClosable={false}
      >
        正在注册...
      </Modal>
      <Modal
        visible={validationStatus === SignupValidationStatus.Valid}
        onOk={onSignupSuccessButtonPressed}
        centered
        okText={"确认"}
        cancelButtonProps={{ style: { display: "none" } }}
      >
        注册成功！
      </Modal>
      <Modal
        visible={validationStatus === SignupValidationStatus.Invalid}
        onOk={onSignupFailedButtonPressed}
        centered
        okText={"确认"}
        cancelButtonProps={{ style: { display: "none" } }}
      >
        注册失败！
      </Modal>
    </React.Fragment>
  );
};

export default Signup;
