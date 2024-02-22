import("../pkg/index.js").then(module =>{
   module.main_web();
});
document.addEventListener('contextmenu', event => {
   event.preventDefault();
});