export function addClass(node, klass) {
  node.classList.add(klass);
}

export function removeClass(node, klass) {
  node.classList.remove(klass);
}

export function setText(node, text) {
  node.textContent = text;
}

export function setHtml(node, html) {
  node.innerHTML = html;
}

export function setDisplay(node, display) {
  node.style.display = display;
}
