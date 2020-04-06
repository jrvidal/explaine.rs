import { addClass, setDisplay, removeClass, setText } from "./util";
import renderer, { pure } from "./renderer";

const querySelector = (selector) => document.querySelector(selector);

export function generateLink({ onAddress, getValue }) {
  const generateButton = querySelector(".generate");
  const generatedLink = querySelector(".link");

  generateButton.addEventListener("click", () => {
    if (getValue() == null) return;
    let address = new window.URL(window.location.href);
    let params = new window.URLSearchParams();
    params.append("code", getValue());
    address.search = `?${params.toString()}`;

    onAddress(address.toString());
  });

  return pure(function renderGeneratedLink({ address, enabled }) {
    generateButton.disabled = !enabled;
    if (address) {
      setDisplay(generateButton, "none");
      removeClass(generatedLink, "hidden");
      generatedLink.href = address;
    } else {
      addClass(generatedLink, "hidden");
      setDisplay(generateButton, null);
    }
  });
}

export function whatsThis() {
  const modal = querySelector(".modal");
  const overlay = querySelector(".overlay");

  let state = {
    showModal: false,
  };

  const setState = renderer(
    (prevState) => {
      if (state.showModal) {
        addClass(modal, "show-modal");
        addClass(overlay, "show-modal");
      } else {
        removeClass(modal, "show-modal");
        removeClass(overlay, "show-modal");
      }
    },
    {
      get() {
        return state;
      },
      set(next) {
        state = next;
      },
    }
  );

  querySelector(".whats-this").addEventListener("click", () => {
    setState(({ showModal }) => ({ showModal: !showModal }));
  });

  overlay.addEventListener("click", () => {
    setState({ showModal: false });
  });

  querySelector(".close-modal").addEventListener("click", () => {
    setState({ showModal: false });
  });
}

export function toggleEdit({ onToggleEdit }) {
  const toggleEditButton = querySelector(".toggle-edit");

  toggleEditButton.addEventListener("click", () => {
    onToggleEdit();
  });

  return pure(({ enabled, editable }) => {
    toggleEditButton.disabled = !enabled;
    setText(toggleEditButton, editable ? "Disable editing" : "Enable editing");
  });
}

export function showAll({ onToggleShowAll }) {
  const showAllButton = querySelector(".show-all");
  const showAllText = querySelector(".show-all-text");
  const showAllSpinner = querySelector(".show-all > .spinner");

  const initialShowAll = showAllText.textContent;

  showAllButton.addEventListener("click", () => {
    onToggleShowAll();
  });

  return pure(function renderShowAll({
    showAll,
    empty,
    canShow,
    failedCompilation,
  }) {
    showAllButton.disabled = !canShow;

    const isLoaded = canShow || failedCompilation || empty;
    (isLoaded ? addClass : removeClass)(showAllButton, "show-all-loaded");

    if (showAll) {
      setText(showAllText, "Hide elements");
    } else {
      setText(showAllText, initialShowAll);
    }
  });
}
