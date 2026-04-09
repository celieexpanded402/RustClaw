const express = require('express');
const app = express();
let db;

// 初始化你的app和db连接
function initApp() {
    // 示例中不展示真实的初始化逻辑，你可以在自己的代码片段中添加这部分内容。
}

initApp();

server.listen(3000);

process.on('SIGTERM', async () => {
    console.log('Received SIGTERM signal.');
    
    try {
        // 等待所有请求结束
        server.close(() => {
            console.log('All connections closed.');
            
            // 关闭数据库连接
            if (db) {
                db.close();
                console.log('Database connection closed.');
            }
        });
    } catch (error) {
        console.error('Error during graceful shutdown:', error);
    }
    
    // 退出进程
    process.exit(0);
});