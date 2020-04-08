export function addClass(node: Element, klass: string) {
  node.classList.add(klass);
}

export function removeClass(node: Element, klass: string) {
  node.classList.remove(klass);
}

export function setText(node: Element, text: string) {
  node.textContent = text;
}

export function setHtml(node: Element, html: string) {
  node.innerHTML = html;
}

export function setDisplay(node: Element, display: string) {
  (node as HTMLElement).style.display = display;
}
