import { addClass, setDisplay, removeClass, setText, makeUrl } from "./util";
import renderer, { pure } from "./renderer";
import { Location } from "./types";

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
    const code = getValue();
    if (code == null) return;
    window.open(
      makeUrl("https://play.rust-lang.org", {
        code,
        edition: "2018",
      }),
      "_blank"
    );
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
  getValue: () => {
    location: Location | null;
    code: string;
  } | null;
}) {
  const generateButton = querySelector(".generate") as HTMLButtonElement;
  const generatedLink = querySelector(".link") as HTMLAnchorElement;

  generateButton.addEventListener("click", () => {
    const { code, location } = getValue() || {};
    if (code == null) return;

    let params: { [param: string]: string } =
      location == null
        ? { code }
        : { code, line: String(location.line), ch: String(location.ch) };

    onAddress(makeUrl(window.location.href, params));
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
    enabled,
  }: {
    showAll: boolean | null | undefined;
    enabled: boolean;
  }) {
    showAllButton.disabled = !enabled;

    (showAll != null ? addClass : removeClass)(
      showAllButton,
      "show-all-loaded"
    );

    if (showAll === true) {
      setText(showAllText, "Hide elements");
    } else {
      setText(showAllText, initialShowAll);
    }
  });
}
