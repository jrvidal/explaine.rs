import renderer, { pure } from "./renderer";
import {
  setDisplay,
  setHtml,
  setText,
  addClass,
  removeClass,
  makeUrl,
} from "./util";
import { CompilationError, Elaboration } from "./index";
import {
  Location,
  MissingHint,
  PENDING,
  ERROR,
  CompilationState,
  SUCCESS,
} from "./types";

const querySelector = (selector: string) => document.querySelector(selector)!!;

function wrapInBlock({ onWrapInBlock }: { onWrapInBlock: () => void }) {
  const canBeBlockEl = querySelector(".can-be-block");

  querySelector(".wrap-in-block").addEventListener("click", () => {
    onWrapInBlock();
  });

  return ({ canBeBlock }: { canBeBlock: boolean }) => {
    setDisplay(canBeBlockEl, canBeBlock ? "block" : "none");
  };
}

const missingTooltip = ({
  onNavigate,
  onOutsideClick,
}: {
  onNavigate: () => void;
  onOutsideClick: () => void;
}) => {
  const missingTooltipEl = querySelector(".missing-tooltip") as HTMLElement;
  const missingTooltipCode = missingTooltipEl.querySelector("code")!!;
  const submitIssueButton = missingTooltipEl.querySelector(
    ".submit-issue"
  ) as HTMLButtonElement;

  let tooltipState = {
    missing: null as MissingHint | null,
  };

  const setTooltipState = renderer<{
    missing: null | MissingHint;
  }>(
    () => {
      renderMissingTooltip({
        missing: tooltipState.missing,
      });
    },
    {
      get() {
        return tooltipState;
      },
      set(next) {
        tooltipState = next;
      },
    }
  );

  submitIssueButton.addEventListener("click", (e) => {
    e.preventDefault();
    const { code, location } = tooltipState.missing || {};
    if (!(code && location)) return;

    const url = makeUrl("https://github.com/jrvidal/explaine.rs/issues/new", {
      labels: "missing-hint",
      title: "Missing Hint",
      body: [
        "### What I expected",
        "<!-- What hint should we show here? What part of this syntax don't you understand? -->",
        "",
        "### Source code",
        "```",
        code,
        "```",
        "",
        `Location: line ${location.line}, column ${location.ch}`,
      ].join("\n"),
    });
    window.open(url, "_blank");
    onNavigate();
  });

  window.addEventListener("click", (e) => {
    if (tooltipState.missing == null) return;
    let el: HTMLElement | null = e.target as HTMLElement;

    do {
      if (el == missingTooltipEl) {
        return;
      }
      el = el.parentElement;
    } while (el != null);

    onOutsideClick();
  });

  function renderMissingTooltip({ missing }: { missing: MissingHint | null }) {
    (missing != null ? addClass : removeClass)(missingTooltipEl, "visible");
    setText(missingTooltipCode, missing?.code ?? "");
  }

  return setTooltipState;
};

function session() {
  const explanationEl = querySelector(".explanation");
  const loadingContainer = explanationEl.querySelector(
    ".loading"
  )!! as HTMLElement;
  const loadedContainer = explanationEl.querySelector(
    ".loaded"
  )!! as HTMLElement;
  const itemContainer = explanationEl.querySelector(".item-container")!!;
  const itemTitle = itemContainer.querySelector(".item-title")!!;
  const itemEl = itemContainer.querySelector(".item")!!;
  const errorMessageContainer = itemContainer.querySelector(
    ".error-message-container"
  )!!;
  const errorMessageEl = itemContainer.querySelector(".error-message")!!;

  const fileBugEl = explanationEl.querySelector(".file-bug")!!;
  const doFileBugLink = explanationEl.querySelector(".do-file-bug")!!;

  const initialItem = itemEl.innerHTML;
  const initialItemTitle = itemTitle.innerHTML;

  let sessionState: State = {
    issueVisible: false,
    error: null,
    compilationState: PENDING,
    elaboration: null,
    missing: null,
  };

  const DIALOG_KEY = "settings.reportDialog";
  let hasSeenReportDialog = localStorage.getItem(DIALOG_KEY) != null;

  type State = Props & { issueVisible: boolean };

  type Props = {
    compilationState: CompilationState;
    error: CompilationError | null;
    elaboration: Elaboration | null;
    missing: MissingHint | null;
  };

  const setSessionState = renderer<State>(
    () => {
      renderSession(sessionState);
    },
    {
      get() {
        return sessionState;
      },
      set(next) {
        sessionState = next;
      },
    }
  );

  const renderMissingTooltip = missingTooltip({
    onNavigate() {
      setSessionState({ issueVisible: false });
    },
    onOutsideClick() {
      setSessionState({ issueVisible: false });
    },
  });

  doFileBugLink.addEventListener("click", (e) => {
    e.preventDefault();
    localStorage.setItem(DIALOG_KEY, "true");
    hasSeenReportDialog = true;
    setSessionState({ issueVisible: !sessionState.issueVisible });
  });

  function renderSession({
    error,
    compilationState,
    elaboration,
    missing,
  }: Props) {
    setDisplay(
      loadingContainer,
      compilationState === PENDING ? "initial" : "none"
    );
    setDisplay(
      loadedContainer,
      compilationState !== PENDING ? "initial" : "none"
    );

    const showFileBug = compilationState === SUCCESS && missing != null;
    setDisplay(fileBugEl, showFileBug ? "block" : "none");
    (showFileBug && !hasSeenReportDialog ? addClass : removeClass)(
      doFileBugLink,
      "shake"
    );

    if (compilationState === ERROR) {
      setHtml(itemTitle, "Oops! 💥");
      setHtml(itemEl, "There is a syntax error in your code:");

      setDisplay(errorMessageContainer, "block");
      setText(errorMessageEl, error!!.msg);
    } else if (elaboration != null) {
      setHtml(itemTitle, elaboration.title);
      setHtml(itemEl, elaboration.elaboration);
      setDisplay(errorMessageContainer, "none");
    } else {
      setHtml(itemTitle, initialItemTitle);
      setHtml(itemEl, initialItem);
      setDisplay(errorMessageContainer, "none");
      renderMissingTooltip({
        missing: sessionState.issueVisible ? sessionState.missing : null,
      });
    }
  }

  return (props: Props) => setSessionState(props);
}

export function aside({ onWrapInBlock }: { onWrapInBlock: () => void }) {
  const explanationEl = querySelector(".explanation");
  const moreInfoHeader = explanationEl.querySelector(".more-info")!!;
  const bookRow = moreInfoHeader.querySelector(".book-row")!!;
  const bookLink = bookRow.querySelector("a")!!;
  const keywordRow = moreInfoHeader.querySelector(".keyword-row")!!;
  const keywordLink = keywordRow.querySelector("a")!!;
  const stdRow = moreInfoHeader.querySelector(".std-row")!!;
  const stdLink = stdRow.querySelector("a")!!;
  const infoWipEl = querySelector(".info-wip");

  const renderWrapInBlock = wrapInBlock({ onWrapInBlock });

  const renderSession = session();

  function renderElaboration({
    elaboration,
  }: {
    elaboration: Elaboration | null;
  }) {
    if (elaboration != null) {
      setDisplay(moreInfoHeader, "block");
      setDisplay(bookRow, elaboration.book ? "block" : "none");
      setDisplay(keywordRow, elaboration.keyword ? "block" : "none");
      setDisplay(stdRow, elaboration.std ? "block" : "none");
      bookLink.href = elaboration.book || "";
      keywordLink.href = elaboration.keyword || "";
      stdLink.href = elaboration.std || "";
      setDisplay(
        infoWipEl,
        elaboration.book || elaboration.keyword || elaboration.std
          ? "none"
          : "initial"
      );
    } else {
      setDisplay(moreInfoHeader, "none");
    }
  }

  const renderWrapInBlockPure = pure(renderWrapInBlock);
  const renderElaborationPure = pure(renderElaboration);
  const renderSessionPure = pure(renderSession);

  return ({
    elaboration,
    error,
    compilationState,
    missing,
  }: {
    compilationState: CompilationState;
    error: CompilationError | null;
    elaboration: Elaboration | null;
    missing: MissingHint | null;
  }) => {
    renderWrapInBlockPure({ canBeBlock: Boolean(error && error.isBlock) });
    renderElaborationPure({ elaboration });
    renderSessionPure({ error, compilationState, elaboration, missing });
  };
}
