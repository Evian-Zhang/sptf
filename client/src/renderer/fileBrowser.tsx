import React, { useEffect, useState } from 'react';
import { createWebsocket, handleWebsocketData, requestChangeDir, downloadFiles, uploadFiles, makeDirectory, logout } from './custom-utils/conn';
import {
  Button,
  Modal,
  PageHeader,
  Row,
  Col,
  Card,
  List,
  Input,
  message
} from 'antd';
import { handleErrorCode } from './custom-utils/error_handling';
import { sptf } from './custom-utils/protos';
import { FileFilled, FolderFilled } from '@ant-design/icons';

export interface FileBrowserProps {
    authToken: string,
    onAuthFailed: () => void,
    onWebsocketFailed: () => void,
}

enum WebsocketConnectStatus {
    Success,
    Failure
}

function FileBrowser(props: FileBrowserProps) {
  const [websocket, setWebsocket] = useState<WebSocket | null>(null);
  const [websocketConnectStatus, setWebsocketConnectStatus] = useState(WebsocketConnectStatus.Success);
  const [currentDirPath, setCurrentDirPath] = useState<string | null>(null);
  const [targetDirPath, setTargetDirPath] = useState("/");
  const [closableError, setClosableError] = useState<string | null>(null);
  const [files, setFiles] = useState<sptf.DirectoryLayout.IFile[]>([]);
  const [selectedIndices, setSelectedIndices] = useState(new Set<number>());
  const [uploading, setUploading] = useState(false); // is file upload modal opened
  const [uploadedFiles, setUploadedFiles] = useState<{fileName: string, content: Blob}[]>([]);
  const [isUploadingFiles, setIsUploadingFiles] = useState(false); // is sending file
  const [creatingDirectory, setCreatingDirectory] = useState(false); // is creating diretcory modal opened
  const [isCreatingDirectory, setIsCreatingDirectory] = useState(false); // is sending creating directory command
  const [newDirectoryName, setNewDirectoryName] = useState("");

  useEffect(() => {
    createWebsocket(props.authToken)
      .then((websocket) => {
        setWebsocket(websocket);
        websocket.onerror = () => {
            setWebsocketConnectStatus(WebsocketConnectStatus.Failure);
        };
        websocket.onmessage = (event) => {
          const data = event.data;
          handleWebsocketData(data)
            .then((listDirectoryResponse) => {
              if (listDirectoryResponse.directoryPath === targetDirPath) {
                if (listDirectoryResponse.DirectoryLayout) {
                  setCurrentDirPath(targetDirPath);
                  setFiles(listDirectoryResponse.DirectoryLayout.files ?? []);
                  setSelectedIndices(new Set());
                } else if (listDirectoryResponse.ErrorResponse) {
                  if (currentDirPath) {
                    setTargetDirPath(currentDirPath);
                  }
                  setClosableError(handleErrorCode(listDirectoryResponse.ErrorResponse.errorCode));
                }
              }
            })
            .catch(() => {
              setWebsocketConnectStatus(WebsocketConnectStatus.Failure);
            })
        };
        setWebsocketConnectStatus(WebsocketConnectStatus.Success);
      })
      .catch(() => {
        setWebsocketConnectStatus(WebsocketConnectStatus.Failure);
      });
  }, []);

  function getFileCover(file: sptf.DirectoryLayout.IFile) {
    if (file.metadata.fileType === sptf.DirectoryLayout.FileMetadata.FileType.NORMAL_FILE) {
      return (<FileFilled style={{fontSize: "80px"}}/>);
    } else {
      return (<FolderFilled style={{fontSize: "80px"}}/>);
    }
  }

  function FileCard(props: {file: sptf.DirectoryLayout.IFile, fileIndex: number, isSelected: boolean}) {
    return (
      <Col lg={4} md={8} sm={12} xs={24}>
        <Card
          size="small"
          style={{backgroundColor: props.isSelected ? "aqua" : "transparent"}}
          onClick={() => {
            setSelectedIndices((selectedIndices) => {
              const selectedIndicesCopy = new Set(selectedIndices);
              if (selectedIndices.has(props.fileIndex)) {
                selectedIndicesCopy.delete(props.fileIndex);
              } else {
                selectedIndicesCopy.add(props.fileIndex);
              }
              return selectedIndicesCopy;
            })
          }}
        >
          <div
            style={{alignContent: "center", textAlign: "center"}}
          >
            {getFileCover(props.file)}<br/>
            {props.file.fileName}
          </div>
        </Card>
      </Col>
    );
  }

  function isSelectedADir(selectedIndex: number) {
    return files[selectedIndex] && files[selectedIndex].metadata.fileType === sptf.DirectoryLayout.FileMetadata.FileType.DIRECTORY;
  }

  return (
    <>
      <Modal
        visible={websocketConnectStatus === WebsocketConnectStatus.Failure}
        onOk={props.onWebsocketFailed}
        centered
        okText={"确认"}
        cancelButtonProps={{ style: { display: "none" } }}
        closable={false}
        maskClosable={false}
      >
        连接失败！
      </Modal>
      <div
        style={{
          width: "100%",
          height: "100%",
          visibility: websocketConnectStatus === WebsocketConnectStatus.Success ? 'visible' : 'hidden'
        }}
      >
        <PageHeader
          title={currentDirPath ?? "正在载入"}
          extra={[
            <Button
              key="goDeeper"
              disabled={selectedIndices.size !== 1 && isSelectedADir(selectedIndices.values().next().value)}
              onClick={() => {
                if (websocket) {
                  const selectedIndex = selectedIndices.values().next().value;
                  const file = files[selectedIndex];
                  setTargetDirPath(file.path);
                  requestChangeDir(websocket, file.path);
                }
              }}
            >
              进入该目录
            </Button>,
            <Button
              key="goUpper"
              disabled={currentDirPath === '/'}
              onClick={() => {
                if (currentDirPath && websocket) {
                  let upperDir = currentDirPath;
                  if (upperDir[upperDir.length - 1] === '/') {
                    upperDir = upperDir.slice(0, upperDir.length - 1)
                  }
                  while (upperDir[upperDir.length - 1] !== '/') {
                    upperDir = upperDir.slice(0, upperDir.length - 1)
                  }
                  upperDir = upperDir.slice(0, upperDir.length - 1)
                  setTargetDirPath(upperDir);
                  requestChangeDir(websocket, upperDir);
                }
              }}
            >
              返回上级目录
            </Button>,
            <Button
              key="refresh"
              onClick={() => {
                if (websocket && currentDirPath) {
                  requestChangeDir(websocket, currentDirPath);
                }
              }}
            >
              刷新
            </Button>,
            <Button
              key="upload"
              onClick={() => {
                setUploading(true);
              }}
            >
              上传
            </Button>,
            <Button
              key="download"
              disabled={selectedIndices.size === 0}
              onClick={() => {
                downloadFiles(props.authToken, files.filter((_, index) => {
                  return selectedIndices.has(index);
                }).map((file) => {
                  return file.path;
                }))
              }}
            >
              下载
            </Button>,
            <Button
              key="makeDirectory"
              disabled={selectedIndices.size === 0}
              onClick={() => {
                setCreatingDirectory(true);
              }}
            >
              创建新目录
            </Button>,
            <Button
              key="logout"
              disabled={selectedIndices.size === 0}
              onClick={() => {
                logout().then(() => {
                  props.onAuthFailed();
                })
              }}
            >
              退出登录
            </Button>,
          ]}
          style={{height: "fitContent", top: 0}}
        />
        <Row gutter={16}>
          {
            files.map((file, fileIndex) => {
              const isSelected = selectedIndices.has(fileIndex);
              return <FileCard key={`${file.path}/${isSelected}`} file={file} fileIndex={fileIndex} isSelected={isSelected}/>;
            })
          }
        </Row>
        <Modal
          visible={closableError !== null}
          onOk={props.onWebsocketFailed}
          centered
          okText={"确认"}
          cancelButtonProps={{ style: { display: "none" } }}
          closable={false}
          maskClosable={false}
        >
          {closableError}
        </Modal>
        <Modal
          visible={targetDirPath !== currentDirPath && websocketConnectStatus === WebsocketConnectStatus.Success}
          centered
          footer={null}
          closable={false}
          maskClosable={false}
        >
          正在连接...
        </Modal>
        <Modal
          visible={uploading}
          closable={isUploadingFiles}
          maskClosable={isUploadingFiles}
          footer={null}
        >
          <List
            dataSource={uploadedFiles}
            renderItem={(uploadedFile) => {
              return (
                <List.Item>
                  {uploadedFile.fileName}
                </List.Item>
              );
            }}
          />
          <Input
            type="file"
            onChange={(event) => {
              const selectedFiles = event.target.files;
              if (selectedFiles) {
                for (let i = 0; i < selectedFiles.length; i++) {
                  const file = selectedFiles.item(i);
                  if (file) {
                    setUploadedFiles((uploadedFiles) => {
                      let uploadedFilesCopy = [...uploadedFiles];
                      uploadedFilesCopy.push({fileName: file.name, content: file})
                      return uploadedFilesCopy;
                    })
                  }
                }
              }
            }}
            multiple
          >
            选择文件以上传
          </Input>
          <Button
            disabled={uploadedFiles.length === 0}
            onClick={() => {
              if (currentDirPath) {
                setIsUploadingFiles(true);
                uploadFiles(currentDirPath, uploadedFiles)
                  .then(() => {
                    setIsUploadingFiles(false);
                    setUploading(false);
                    setUploadedFiles([]);
                  })
                  .catch((reason) => {
                    message.error(reason);
                    setIsUploadingFiles(false);
                    setUploading(false);
                    setUploadedFiles([]);
                  })
              }
            }}
          >
            上传
          </Button>
        </Modal>
        <Modal
          visible={creatingDirectory}
          closable={isCreatingDirectory}
          maskClosable={isCreatingDirectory}
          footer={null}
        >
          <Input
            placeholder="请输入目录名"
            onChange={(event) => {
              const dirName = event.target.value;
              let directoryPath = "";
              if (currentDirPath) {
                if (currentDirPath.endsWith('/')) {
                  directoryPath = `${currentDirPath}${dirName}`;
                } else {
                  directoryPath = `${currentDirPath}/${dirName}`;
                }
              }
              setNewDirectoryName(directoryPath);
            }}
          />
          <Button
            onClick={() => {
              makeDirectory(newDirectoryName)
                .then(() => {
                  setIsCreatingDirectory(false);
                  setCreatingDirectory(false);
                  setNewDirectoryName("");
                })
                .catch((reason) => {
                  message.error(reason);
                  setIsCreatingDirectory(false);
                  setCreatingDirectory(false);
                  setNewDirectoryName("");
                });
            }}
          >
            确认
          </Button>
        </Modal>
      </div>
    </>
  );
}

export default FileBrowser;
