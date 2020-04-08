import { addClass, setDisplay, removeClass, setText } from "./util";
import renderer, { pure } from "./renderer";

const querySelector = (selector: string) => document.querySelector(selector)!!;

export function openInPlayground({
  getValue,
}: {
  getValue: () => string | null;
}) {
  const openInPlaygroundButton = querySelector(
    ".playground"
  ) as HTMLButtonElement;

  openInPlaygroundButton.addEventListener("click", () => {
    const value = getValue();
    if (value == null) return;
    window.open(codeAsSearchUrl("https://play.rust-lang.org", value), "_blank");
  });

  return pure(function renderOpenInPlayground({
    enabled,
  }: {
    enabled: boolean;
  }) {
    openInPlaygroundButton.disabled = !enabled;
  });
}

export function generateLink({
  onAddress,
  getValue,
}: {
  onAddress: (address: string) => void;
  getValue: () => string | null;
}) {
  const generateButton = querySelector(".generate") as HTMLButtonElement;
  const generatedLink = querySelector(".link") as HTMLAnchorElement;

  generateButton.addEventListener("click", () => {
    const value = getValue();
    if (value == null) return;

    onAddress(codeAsSearchUrl(window.location.href, value));
  });

  return pure(function renderGeneratedLink({
    address,
    enabled,
  }: {
    address: string | null;
    enabled: boolean;
  }) {
    generateButton.disabled = !enabled;
    if (address) {
      setDisplay(generateButton, "none");
      removeClass(generatedLink, "hidden");
      generatedLink.href = address;
    } else {
      setDisplay(generateButton, "initial");
      addClass(generatedLink, "hidden");
    }
  });
}

export function whatsThis() {
  const modal = querySelector(".modal");
  const overlay = querySelector(".overlay");

  let state: { showModal: boolean } = {
    showModal: false,
  };

  const setState = renderer<{ showModal: boolean }>(
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

export function toggleEdit({ onToggleEdit }: { onToggleEdit: () => void }) {
  const toggleEditButton = querySelector(".toggle-edit") as HTMLButtonElement;

  toggleEditButton.addEventListener("click", () => {
    onToggleEdit();
  });

  return pure(
    ({ enabled, editable }: { enabled: boolean; editable: boolean }) => {
      toggleEditButton.disabled = !enabled;
      setText(
        toggleEditButton,
        editable ? "Disable editing" : "Enable editing"
      );
    }
  );
}

export function showAll({ onToggleShowAll }: { onToggleShowAll: () => void }) {
  const showAllButton = querySelector(".show-all") as HTMLButtonElement;
  const showAllText = querySelector(".show-all-text");

  const initialShowAll = showAllText.textContent!!;

  showAllButton.addEventListener("click", () => {
    onToggleShowAll();
  });

  return pure(function renderShowAll({
    showAll,
    empty,
    canShow,
    failedCompilation,
  }: {
    showAll: boolean;
    empty: boolean;
    canShow: boolean;
    failedCompilation: boolean;
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

function codeAsSearchUrl(url: string, code: string) {
  let address = new window.URL(url);
  let params = new window.URLSearchParams();
  params.append("code", code);
  address.search = `?${params.toString()}`;
  return address.toString();
}
