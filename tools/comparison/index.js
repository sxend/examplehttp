const cluster = require('cluster');
const http = require('http');
const numCPUs = 4;
if (cluster.isMaster) {
    for (let i = 0; i < numCPUs; i++) {
        cluster.fork();
    }
} else {
    http.createServer((req, res) => {
        res.writeHead(200, { 'Content-Type': 'text/plain' });
        res.end(`hi. process on ${cluster.worker.id}`);
    }).listen(8889);
}