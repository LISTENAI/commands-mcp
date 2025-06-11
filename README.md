commands-mcp
============

使用命令模板构建 MCP Server。

## 安装

```json
{
  "mcpServers": {
    "commands":{
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "-v", "/path/to/your/commands.yaml:/commands.yaml:ro",
        "ghcr.io/listenai/commands-mcp:master",
        "/commands.yaml"
      ]
    }
  }
}
```

## 示例

```yaml
# commands.yaml
---
zephyr_build:
  description: 编译当前 Zephyr 项目
  args:
    - name: board
      description: 需要构建的 board identifier，如果不能从对话历史中确定，则询问用户
      type: string
      required: true
    - name: source_dir
      description: 项目源码目录，如未指定则使用当前目录
      type: string
  command: source .venv/bin/activate; west build -p -s {source_dir} -b {board}

zephyr_flash:
  description: 将编译好的固件烧录到设备
  command: west flash
```

## 协议

[Apache-2.0](LICENSE)
