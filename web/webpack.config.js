const webpack = require("webpack");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const OptimizeCSSAssetsPlugin = require("optimize-css-assets-webpack-plugin");
const env = require("../env.json");

module.exports = {
  entry: {
    index: "./web/index.js",
    style: "./web/style.css",
  },
  output: {
    filename: "[name]-[contenthash].js",
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, "css-loader"],
      },
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: "[name]-[contenthash].css",
    }),
    new OptimizeCSSAssetsPlugin(),
    new webpack.ProvidePlugin({
      CodeMirror: require.resolve("codemirror/lib/codemirror.js"),
    }),
    new webpack.DefinePlugin({
      "self.SKIP_LOGGING": JSON.stringify(true),
      __ANALYTICS_URL__: JSON.stringify(
        env.analyticsUrl ||
          (() => {
            throw new Error("No analyticsUrl");
          })()
      ),
      __PRODUCTION__: JSON.stringify(true),
      "self.IS_WORKER": JSON.stringify(false),
    }),
  ],
};
