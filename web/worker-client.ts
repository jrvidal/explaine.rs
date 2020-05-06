import AnalyzerWorker from "worker-loader!./worker.js";
import { READY, SECONDARY_LOAD, MAIN_LOAD } from "./messages";
import { logInfo, reportError } from "./logging";
import { defer } from "./util";

export default function worker({
  onMessage,
}: {
  onMessage: (message: any) => void;
}) {
  const mainWorker: Worker = new AnalyzerWorker();
  const secondaryWorker: Worker = new AnalyzerWorker();

  let {
    promise: mainWorkerIsReadyPromise,
    resolve: resolveMainWorkerIsReady,
  } = defer();
  let {
    promise: secondaryWorkerIsReadyPromise,
    resolve: resolveSecondaryWorkerIsReady,
  } = defer();

  let workerIsReadyPromise = Promise.all([
    mainWorkerIsReadyPromise,
    secondaryWorkerIsReadyPromise,
  ]);

  mainWorker.onerror = (e) => reportError("mainworker.onerror", e);
  ((mainWorker as any) as MessagePort).onmessageerror = (e) =>
    reportError("mainworker.onmessageerror", e);
  secondaryWorker.onerror = (e) => reportError("secondaryworker.onerror", e);
  ((secondaryWorker as any) as MessagePort).onmessageerror = (e) =>
    reportError("secondaryworker.onmessageerror", e);

  mainWorker.onmessage = (e) => {
    const { data } = e;
    logInfo("Window received", data.type, data);
    if (data.type === READY) {
      resolveMainWorkerIsReady(null);
      secondaryWorker.postMessage({
        type: SECONDARY_LOAD,
        compiledModule: data.compiledModule,
      });
      return;
    }
    onMessage(data);
  };

  secondaryWorker.onmessage = (e) => {
    const { data } = e;
    logInfo("Window received", data.type, data);
    if (data.type === READY) {
      resolveSecondaryWorkerIsReady(null);
      return;
    }
    onMessage(data);
  };

  if (!self.__PRODUCTION__) {
    (window as any).worker = mainWorker;
  }

  mainWorker.postMessage({
    type: MAIN_LOAD,
  });

  return {
    postMessage: (data: any) => {
      mainWorker.postMessage(data);
      secondaryWorker.postMessage(data);
    },
    ready: workerIsReadyPromise,
  };
}
