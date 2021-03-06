import React, { useEffect, useState } from 'react';
import { createWebsocket, handleWebsocketData, requestChangeDir, downloadFiles } from './custom-utils/conn';
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
  const [uploadedFiles, setUploadedFiles] = useState<{fileName: string, path: string}[]>([]);
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
          console.log("Message got");
          const data = event.data;
          handleWebsocketData(data)
            .then((listDirectoryResponse) => {
              console.log(`Directory path: ${listDirectoryResponse.directoryPath}`)
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
        requestChangeDir(websocket, targetDirPath);
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
        onOk={() => {
          if (websocket) {
            websocket.close();
          }
          props.onWebsocketFailed();
        }}
        centered
        okText={"??????"}
        cancelButtonProps={{ style: { display: "none" } }}
        closable={false}
        maskClosable={false}
      >
        ???????????????
      </Modal>
      <div
        style={{
          width: "100%",
          height: "100%",
          visibility: websocketConnectStatus === WebsocketConnectStatus.Success ? 'visible' : 'hidden'
        }}
      >
        <PageHeader
          title={currentDirPath ?? "????????????"}
          extra={[
            <Button
              key="goDeeper"
              disabled={selectedIndices.size !== 1 || !isSelectedADir(selectedIndices.values().next().value)}
              onClick={() => {
                if (websocket) {
                  const selectedIndex = selectedIndices.values().next().value;
                  const file = files[selectedIndex];
                  setTargetDirPath(file.path);
                  requestChangeDir(websocket, file.path);
                }
              }}
            >
              ???????????????
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
              ??????????????????
            </Button>,
            <Button
              key="refresh"
              onClick={() => {
                if (websocket && currentDirPath) {
                  requestChangeDir(websocket, currentDirPath);
                }
              }}
            >
              ??????
            </Button>,
            <Button
              key="upload"
              onClick={() => {
                setUploading(true);
              }}
            >
              ??????
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
              ??????
            </Button>,
            <Button
              key="makeDirectory"
              disabled={selectedIndices.size !== 0}
              onClick={() => {
                setCreatingDirectory(true);
              }}
            >
              ???????????????
            </Button>,
            <Button
              key="logout"
              onClick={() => {
                window.sptfAPI.logout().then(() => {
                  if (websocket) {
                    websocket.close();
                  }
                  props.onAuthFailed();
                })
              }}
            >
              ????????????
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
          onOk={() => {
            if (websocket) {
              websocket.close();
            }
            props.onWebsocketFailed();
          }}
          centered
          okText={"??????"}
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
          ????????????...
        </Modal>
        <Modal
          visible={uploading}
          closable={!isUploadingFiles}
          maskClosable={!isUploadingFiles}
          footer={null}
          onCancel={() => {
            setUploading(false);
          }}
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
                      return [...uploadedFiles, {fileName: file.name, path: file.path}];
                    });
                  }
                }
              }
            }}
            multiple
          />
          <Button
            disabled={uploadedFiles.length === 0}
            onClick={() => {
              if (currentDirPath) {
                setIsUploadingFiles(true);
                window.sptfAPI.uploadFiles(currentDirPath, uploadedFiles)
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
            ??????
          </Button>
        </Modal>
        <Modal
          visible={creatingDirectory}
          closable={!isCreatingDirectory}
          maskClosable={!isCreatingDirectory}
          onCancel={() => {
            setCreatingDirectory(false);
          }}
          footer={null}
        >
          <Input
            placeholder="??????????????????"
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
              window.sptfAPI.makeDirectory(newDirectoryName)
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
            ??????
          </Button>
        </Modal>
      </div>
    </>
  );
}

export default FileBrowser;
