import AnalyzerWorker from "worker-loader!./worker.js";
import { READY } from "./messages";
import { logInfo, reportError } from "./logging";

export default function worker({
  onMessage,
}: {
  onMessage: (message: any) => void;
}) {
  const worker: Worker = new AnalyzerWorker();

  let resolveWorkerIsReady: () => void;

  let workerIsReadyPromise = new Promise((res) => {
    resolveWorkerIsReady = res;
  });

  worker.onerror = (e) => reportError("worker.onerror", e);

  ((worker as any) as MessagePort).onmessageerror = (e) =>
    reportError("onmessageerror", {
      error: e,
    });

  worker.onmessage = (e) => {
    const { data } = e;
    logInfo("Window received", data.type, data);
    if (data.type === READY) {
      resolveWorkerIsReady();
      return;
    }
    onMessage(data);
  };

  if (!self.__PRODUCTION__) {
    (window as any).worker = worker;
  }

  return {
    postMessage: (data: any) => worker.postMessage(data),
    ready: workerIsReadyPromise,
  };
}
