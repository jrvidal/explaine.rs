const path = require("path");

module.exports = {
  entry: "./web/index.ts",
  output: {
    globalObject: "this",
  },
  resolve: {
    extensions: ["wasm", ".mjs", ".js", ".json", ".ts"],
  },
  module: {
    rules: [
      { test: /\.ts$/, loader: "ts-loader" },
      {
        test: require.resolve("../pkg/explainers.js"),
        use: "exports-loader?wasm_bindgen",
      },
      {
        test: /\.wasm$/,
        type: "javascript/auto",
        use: [
          {
            loader: "file-loader",
          },
        ],
      },
    ],
  },
  devtool: "none",
  devServer: {
    host: "0.0.0.0",
    contentBase: path.join(__dirname, ".."),
    index: "",
    proxy: {
      "/": {
        target: "http://0.0.0.0:8080/web",
        bypass: (req) => {
          if (req.path !== "/") {
            return req.path;
          }
        },
      },
    },
  },
};
