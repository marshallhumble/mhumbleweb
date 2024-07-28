const encEmail = "bWh1bWJsZUBtaHVtYmxlLmlv";

// Correct: Wait for the DOM to load before modifying the element
document.addEventListener('DOMContentLoaded', function() {
    const ele = document.getElementById("contact");
    if (ele) {
        ele.setAttribute("href", "mailto:".concat(atob(encEmail)));
    } else {
        console.error("Element with ID 'my-element' not found.");
    }
});