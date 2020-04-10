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

export function makeUrl(url: string, params: { [param: string]: string }) {
  let address = new window.URL(url);
  let searchParams = new window.URLSearchParams();
  Object.entries(params).forEach(([key, param]) => {
    searchParams.append(key, param);
  });
  address.search = `?${params.toString()}`;
  return address.toString();
}
