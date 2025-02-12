// /static/js/modal.js

/**
 * Shows the modal with the given description.
 */
export function showModal(description) {
  const modal = document.getElementById("pylonModal");
  const modalDescription = document.getElementById("modalDescription");
  modalDescription.innerText = description;
  modal.style.display = "block";
}

/**
 * Hides the modal.
 */
export function hideModal() {
  const modal = document.getElementById("pylonModal");
  modal.style.display = "none";
}
