module.exports = {
  entry: "./web/index.js",
  output: {
    globalObject: "this",
  },
  module: {
    rules: [
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
};
