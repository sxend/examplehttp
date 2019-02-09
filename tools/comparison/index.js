const cluster = require('cluster');
const http = require('http');
const numCPUs = 4;
if (cluster.isMaster) {
    for (let i = 0; i < numCPUs; i++) {
        cluster.fork();
    }
} else {
    http.createServer((req, res) => {
        setTimeout(() => {
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify(msg(req), null, "  "));
        }, 5000)
    }).listen(8889);
}
function msg(req) {
    return {
        request: {
            version: req.httpVersion,
            method: req.method,
            path: req.url,
            headers: Object.entries(req.headers).map(x => {
                return {
                    name: x[0],
                    value: x[1]
                };
            })
        },
        ext: {
            process_thread: cluster.worker && cluster.worker.id
        }
    };
}