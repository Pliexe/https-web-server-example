import express from "express";
import http from "http";
import https from "https";
import path from "path";
import fs from "fs";
import livereload from "livereload";
import connectLivereload from "connect-livereload";

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

app.get('/:filename', function(req, res) {
    let filename = req.params.filename;
    if (!filename.includes(".")) filename += ".html";
    
    res.sendFile(filename, { root: path.join(__dirname, `../public`) }, function(err) {
        if (err) {
            res.sendFile(path.join(__dirname, "../public/404.html"));
        }
    });
});

app.use(express.static(path.join(__dirname, "../public")));

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
})