#!/usr/bin/env node

const { minify: minifyHtml } = require('html-minifier-terser');
const { minify: minifyJs } = require('terser');
const CleanCSS = require('clean-css');
const MarkdownIt = require('markdown-it');
const fs = require('fs');
const path = require('path');

// 配置选项
const options = {
  collapseWhitespace: true,
  removeComments: true,
  removeEmptyAttributes: true,
  removeOptionalTags: true,
  removeRedundantAttributes: true,
  removeScriptTypeAttributes: true,
  removeStyleLinkTypeAttributes: true,
  minifyCSS: true,
  minifyJS: true,
  processScripts: ['application/json'],
};

// CSS 压缩选项
const cssOptions = {
  level: 2
};

// 处理文件
async function minifyFile(inputPath, outputPath) {
  try {
    let ext = path.extname(inputPath).toLowerCase();
    if (ext === '.md') ext = '.html';
    const filename = path.basename(inputPath);
    let content = fs.readFileSync(inputPath, 'utf8');
    let minified;

    // 特殊处理 readme.html
    if (filename.toLowerCase() === 'readme.md') {
      const md = new MarkdownIt({
        html: true,
        linkify: true,
        typographer: true
      });
      const readmeMdPath = path.join(__dirname, '..', 'README.md');
      const markdownContent = fs.readFileSync(readmeMdPath, 'utf8');
      // 添加基本的 markdown 样式
      const htmlContent = `
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>README</title>
    <style>
        body {
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            line-height: 1.6;
        }
        pre {
            background-color: #f6f8fa;
            padding: 16px;
            border-radius: 6px;
            overflow: auto;
        }
        code {
            background-color: #f6f8fa;
            padding: 0.2em 0.4em;
            border-radius: 3px;
        }
        img {
            max-width: 100%;
        }
        table {
            border-collapse: collapse;
            width: 100%;
        }
        table td, table th {
            border: 1px solid #dfe2e5;
            padding: 6px 13px;
        }
        blockquote {
            border-left: 4px solid #dfe2e5;
            margin: 0;
            padding: 0 1em;
            color: #6a737d;
        }
    </style>
</head>
<body>
    ${md.render(markdownContent)}
</body>
</html>
      `;
      content = htmlContent;
    }

    switch (ext) {
      case '.html':
        minified = await minifyHtml(content, options);
        break;
      case '.js':
        const result = await minifyJs(content);
        minified = result.code;
        break;
      case '.css':
        minified = new CleanCSS(cssOptions).minify(content).styles;
        break;
      default:
        throw new Error(`Unsupported file type: ${ext}`);
    }

    fs.writeFileSync(outputPath, minified);
    console.log(`✓ Minified ${path.basename(inputPath)} -> ${path.basename(outputPath)}`);
  } catch (err) {
    console.error(`✗ Error processing ${inputPath}:`, err);
    process.exit(1);
  }
}

// 主函数
async function main() {
  // 获取命令行参数，跳过前两个参数（node和脚本路径）
  const files = process.argv.slice(2);

  if (files.length === 0) {
    console.error('No input files specified');
    process.exit(1);
  }

  const staticDir = path.join(__dirname, '..', 'static');

  for (const file of files) {
    // 特殊处理 README.md 的输入路径
    let inputPath;
    let outputPath;

    if (file.toLowerCase() === 'readme.md') {
      inputPath = path.join(__dirname, '..', 'README.md');
      outputPath = path.join(staticDir, 'readme.min.html');
    } else {
      inputPath = path.join(staticDir, file);
      const ext = path.extname(file);
      outputPath = path.join(
        staticDir,
        file.replace(ext, `.min${ext}`)
      );
    }

    await minifyFile(inputPath, outputPath);
  }
}

main();