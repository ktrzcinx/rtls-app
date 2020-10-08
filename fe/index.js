import * as rtls from "rtls";

console.log("start");

const dev_canvas = document.getElementById("device-map");
dev_canvas.height = 640
dev_canvas.width = 640
const grid_resolution = 50

const ctx = dev_canvas.getContext('2d');

const zone = rtls.init();
zone.add_device(1, 15, 30, 0);
zone.add_device(2, 300, 270, 0);
zone.add_measure(1, 2, 200, 123);
zone.add_measure(1, 2, 220, 127);

var map = {zoom:0.5, rotation:0.0, offset:[0,dev_canvas.height], moving:false};

dev_canvas.onwheel = (event) => {
        let real_pointing_pos = pixelsToPos([event.clientX, event.clientY]);
        map.zoom += -10 * map.zoom / (event.deltaY + map.zoom);
        map.zoom = Math.min(Math.max(0.05, map.zoom), 10);
        let new_pointing_pixels = posToPixels(real_pointing_pos);
        map.offset[0] += -new_pointing_pixels[0] + event.clientX;
        map.offset[1] += -new_pointing_pixels[1] + event.clientY;
}
dev_canvas.onmousedown = () => {
        map.moving = true;
}
document.body.onmouseup = () => {
        map.moving = false;
}
dev_canvas.onmousemove = (event) => {
        if (map.moving) {
                map.offset[0] += event.movementX;
                map.offset[1] += event.movementY;
        }
}

function getLegendResolition() {
        let resolution_in_pixels = 10;
        let range = dev_canvas.width / map.zoom / resolution_in_pixels;
        let exponent = Math.floor(Math.log10(range));
        let fraction = range / Math.pow(10, exponent);
        var niceFraction = 0;

        if (fraction < 1.5)
                niceFraction = 1;
        else if (fraction < 3)
                niceFraction = 2;
        else if (fraction < 7)
                niceFraction = 5;
        else
                niceFraction = 10;
        let real_resolution = niceFraction * Math.pow(10, exponent);
        let pixels_resolution = real_resolution * map.zoom;
        return {real: real_resolution, pixels: pixels_resolution};
}

function getStartOffsets(offset_pix, resolution) {
        let off_real = offset_pix / map.zoom;
        let off_real_n = Math.floor((-off_real) / resolution["real"]) * resolution["real"];
        let off_pix = offset_pix % resolution["pixels"];
        if (off_pix <= 0) {
                off_pix += resolution["pixels"];
        }
        return {real: off_real_n, pixels: off_pix};
}

const drawGrid = () => {
        ctx.save();
        ctx.beginPath();
        ctx.strokeStyle = "#EEEEEE";
        ctx.font = "12px Arial";
        ctx.textAlign = "left";
        let resolution = getLegendResolition();
        let offset = [getStartOffsets(map.offset[0], resolution),
                      getStartOffsets(map.offset[1], resolution)];
        // draw grid
        for (let col = 0; col < dev_canvas.width / resolution["pixels"]; col += 1) {
                let col_px = col * resolution["pixels"] + offset[0]["pixels"]
                let label = Math.round(pixelsToPos([col_px, 0])[0] / resolution["real"]) * resolution["real"];
                ctx.moveTo(col_px, 0);
                ctx.lineTo(col_px, dev_canvas.height);
                ctx.fillText(label, col_px, dev_canvas.height - 6);
        }
        for (let row = 0; row < dev_canvas.height / resolution["pixels"]; row += 1) {
                let row_px = row * resolution["pixels"] + offset[1]["pixels"]
                let label = Math.round(pixelsToPos([0, row_px])[1] / resolution["real"]) * resolution["real"]
                ctx.moveTo(0, row_px);
                ctx.lineTo(dev_canvas.width, row_px);
                ctx.fillText(label, 0, row_px);
        }
        // Draw border
        ctx.moveTo(0, 0);
        ctx.lineTo(dev_canvas.width, 0);
        ctx.lineTo(dev_canvas.width, dev_canvas.height);
        ctx.lineTo(0, dev_canvas.height);
        ctx.lineTo(0, 0);

        ctx.stroke();
        ctx.restore();
}

function get_img(id)
{
        if (typeof get_img.list == 'undefined') {
                get_img.list = [];
                let path = "/icons/avatars/";
                var names = ["m1.svg", "m2.svg", "w1.svg", "w2.svg"];
                names.forEach((elem) => {
                        let img = new Image();
                        img.src = path + elem;
                        get_img.list.push(img);
                });
        }
        return get_img.list[id % get_img.list.length];
}

function posToPixels(pos) {
        var np = [pos[0], -pos[1]]; // invert Y
         // zoom and translation
        np = [np[0] * map.zoom + map.offset[0],
              np[1] * map.zoom + map.offset[1]];
        return np;
}

function pixelsToPos(px) {
        var pos = [px[0], px[1]];
         // translation and zoom
        pos = [(pos[0] - map.offset[0]) / map.zoom,
               (pos[1] - map.offset[1]) / map.zoom];
        pos = [pos[0], -pos[1]]; // invert Y
        return pos;
}

function stickToBorders(pos, border) {
        var real_position = true;
        var np = [pos[0], pos[1]];
        if (border > 0) {
                np[0] = Math.max(border/2, Math.min(dev_canvas.width-border/2, np[0]));
                np[1] = Math.max(border, Math.min(dev_canvas.height-border, pos[1]));
        }
        if (pos[0] != np[0] || pos[1] != np[1])
                real_position = false;
        return [np, real_position];
}

const drawDevices = () => {
        const SIZE = 50
        var date = new Date();
        let devs = zone.get_all_devices_position(date.getTime());
        ctx.save();
        ctx.fillStyle = "#121540";
        ctx.font = "25px Arial";
        ctx.textAlign = "center";

        devs.forEach((elem) => {
                let pixels_pos = posToPixels(elem.pos.cord);
                let [pos, real_position] = stickToBorders(pixels_pos, SIZE);
                if (real_position)
                        ctx.globalAlpha = 1.0;
                else
                        ctx.globalAlpha = 0.7;
                ctx.drawImage(get_img(elem.id), pos[0] - SIZE / 2, pos[1] - SIZE, SIZE, SIZE);
                ctx.fillText(elem.id, pos[0], pos[1] + 25);
                ctx.fillText(Math.round(elem.pos.cord[0]) + ":" + Math.round(elem.pos.cord[1]), pos[0], pos[1] + 45);
        });
        ctx.restore();
}

const renderLoop = () => {
        ctx.clearRect(0, 0, dev_canvas.width, dev_canvas.height); // clear canvas
        drawGrid();
        drawDevices()

        zone.add_measure(1, 2, 220, 127);
        zone.add_measure(1, 0, 220, 127);
        requestAnimationFrame(renderLoop);
};

const test_code = () => {
        let dev_ptr = zone.get_device_ptr(0);
        let dev = rtls.device_serialize(dev_ptr);
        console.log('dev:', dev);
        let pos = dev.trace[0].cord
        console.log(pos)
}

test_code()
renderLoop()