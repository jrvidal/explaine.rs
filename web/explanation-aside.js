import { pure } from "./renderer";
import { setDisplay, setHtml, setText } from "./util";
import { PENDING, ERROR } from "./index";

const querySelector = (selector) => document.querySelector(selector);

function wrapInBlock({ onWrapInBlock }) {
  const canBeBlockEl = querySelector(".can-be-block");

  querySelector(".wrap-in-block").addEventListener("click", () => {
    onWrapInBlock();
  });

  return ({ canBeBlock }) => {
    setDisplay(canBeBlockEl, canBeBlock ? "block" : "none");
  };
}

export function aside({ onWrapInBlock }) {
  const explanationEl = querySelector(".explanation");
  const loadingContainer = explanationEl.querySelector(".loading");
  const loadedContainer = explanationEl.querySelector(".loaded");
  const itemContainer = explanationEl.querySelector(".item-container");
  const itemTitle = itemContainer.querySelector(".item-title");
  const itemEl = itemContainer.querySelector(".item");
  const errorMessageContainer = itemContainer.querySelector(
    ".error-message-container"
  );
  const errorMessageEl = itemContainer.querySelector(".error-message");
  const moreInfoHeader = explanationEl.querySelector(".more-info");
  const bookRow = moreInfoHeader.querySelector(".book-row");
  const bookLink = bookRow.querySelector("a");
  const keywordRow = moreInfoHeader.querySelector(".keyword-row");
  const keywordLink = keywordRow.querySelector("a");
  const infoWipEl = querySelector(".info-wip");

  const initialItem = itemEl.innerHTML;
  const initialItemTitle = itemTitle.innerHTML;

  const renderWrapInBlock = wrapInBlock({ onWrapInBlock });

  function renderSessionState({ error, compilationState, elaboration }) {
    loadingContainer.style.display =
      compilationState === PENDING ? "initial" : "none";
    loadedContainer.style.display =
      compilationState !== PENDING ? "initial" : "none";

    if (compilationState === ERROR) {
      setHtml(itemTitle, "Oops! 💥");
      setHtml(itemEl, "There is a syntax error in your code:");

      setDisplay(errorMessageContainer, "block");
      setText(errorMessageEl, error.msg);
    } else if (elaboration != null) {
      setHtml(itemTitle, elaboration.title);
      setHtml(itemEl, elaboration.elaboration);
      setDisplay(errorMessageContainer, "none");
    } else {
      setHtml(itemTitle, initialItemTitle);
      setHtml(itemEl, initialItem);
      setDisplay(errorMessageContainer, "none");
    }
  }

  function renderElaboration({ elaboration }) {
    if (elaboration != null) {
      setDisplay(moreInfoHeader, "block");
      setDisplay(bookRow, elaboration.book ? "block" : "none");
      setDisplay(keywordRow, elaboration.keyword ? "block" : "none");
      bookLink.href = elaboration.book || "";
      keywordLink.href = elaboration.keyword || "";
      setDisplay(
        infoWipEl,
        elaboration.book || elaboration.keyword ? "none" : "initial"
      );
    } else {
      setDisplay(moreInfoHeader, "none");
    }
  }

  const renderWrapInBlockPure = pure(renderWrapInBlock);
  const renderElaborationPure = pure(renderElaboration);
  const renderSessionStatePure = pure(renderSessionState);

  return ({ elaboration, error, compilationState }) => {
    renderWrapInBlockPure({ canBeBlock: error && error.isBlock });
    renderElaborationPure({ elaboration });
    renderSessionStatePure({ error, compilationState, elaboration });
  };
}
