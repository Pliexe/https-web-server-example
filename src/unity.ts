import express from "express";
import http from "http";
import https from "https";
import path from "path";
import fs from "fs";
import livereload from "livereload";
import connectLivereload from "connect-livereload";
import { exec } from "child_process";
import expressStaticGzip from "express-static-gzip";

const app = express();
const httpConnection = http.createServer(app);
const httpsConnection = https.createServer({
    key: fs.readFileSync(path.join(__dirname, "../certs/localhost-key.pem")),
    cert: fs.readFileSync(path.join(__dirname, "../certs/localhost.pem")),
}, app);

const HTTP_PORT = 80;
const HTTPS_PORT = 443;

const liveReloadServer = livereload.createServer({ https: httpsConnection });

liveReloadServer.watch(path.join(__dirname, "../public"));

app.use(connectLivereload());

// Serve static files with Brotli compression
app.use(expressStaticGzip(path.join(__dirname, "../public"), {
    enableBrotli: true,
    orderPreference: ['br', 'gz'],
    serveStatic: {
        setHeaders: function(res, path) {
            if (/\.br$/.test(path)) {
                res.setHeader('Content-Encoding', 'br');
            }

            if (/\.gz$/.test(path)) {
                res.setHeader('Content-Encoding', 'gzip');
            }

            if (/\wasm.br$/.test(path) || /\.gz$/.test(path)) {
                res.setHeader('Content-Type', 'application/wasm');
            }
        }
    }
}));

// app.use(express.static(path.join(__dirname, "../public")));

app.use((err: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
    console.error(err);
    res.send(`
        <h1>Error</h1>
        <p>${err.message}</p>
    `);
});

httpConnection.listen(HTTP_PORT, () => {
    console.log(`HTTP server listening on port ${HTTP_PORT}`);
});
httpsConnection.listen(HTTPS_PORT, () => {
    console.log(`HTTPS server listening on port ${HTTPS_PORT}`);
});

const url = "https://localhost";

// Open https://localhost in the default browser
// For Windows
if (process.platform === 'win32') {
    exec(`start ${url}`);
}
// For macOS/Linux
else {
    exec(`open -a "Google Chrome" "${url}"`);
}
