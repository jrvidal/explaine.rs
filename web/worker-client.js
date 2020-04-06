import Worker from "worker-loader!./worker.js";
import { READY } from "./messages";
import { logInfo } from "./logging";

export default function worker({ onMessage }) {
  const worker = new Worker();

  let resolveWorkerIsReady;

  let workerIsReadyPromise = new Promise((res) => {
    resolveWorkerIsReady = res;
  });

  worker.onerror = (e) => console.error(e);

  worker.onmessage = (e) => {
    const { data } = e;
    logInfo("Window received", data.type, data);
    if (data.type === READY) {
      resolveWorkerIsReady();
      return;
    }
    onMessage(data);
  };

  return {
    postMessage: (data) => worker.postMessage(data),
    ready: workerIsReadyPromise,
  };
}
